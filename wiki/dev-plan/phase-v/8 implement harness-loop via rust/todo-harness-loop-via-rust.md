# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача на разработку: перенос harness-loop с Python на чистый Rust
Реализовать нативный Rust-бинарник `harness.exe`, эквивалентный текущему Python-harness (`tools/harness`) по поведению agent-loop, с сохранением базовой архитектуры и тремя целевыми изменениями:

1. Tools — отдельные подключаемые модули, классифицированные по категориям.
2. Отдельный модуль чтения конфигурации; основной файл — `./harness/harness.json5`.
3. Запуск только через CLI `harness.exe` (не через `invoke`).

Python-код (`tools/harness/**`) — эталон поведения. Его не менять и не удалять.

Зачем: уйти от Python/invoke-обвязки к одному нативному бинарнику; разнести tools по категориям; сделать конфигурацию и CLI явным слоем, а не словарями внутри `invoke.task`.

Ожидаемый результат:
- Cargo-проект, собирающий `harness.exe`.
- `harness.exe --help` печатает справку.
- `harness.exe --prompt "..."` стартует новую сессию по конфигу.
- `harness.exe --dump <path>` продолжает сессию из dump-файла.
- Поведение loop / sandbox / tools / лимитов / логов совпадает с Python-эталоном (кроме формата dump: совместимость с Python-dump не требуется).

## Функциональные требования

### 1. Расположение и границы
- Новый crate: `tools/harness-rs/` (binary name: `harness`).
- Эталон (read-only): `tools/harness/**`, `harness/harness.json5`, `harness/grok-4.6.json5`.
- Python не модифицировать.
- `harness/*.json5` менять только если текущей схемы не хватает для обязательных полей; любое расширение — минимальное, описать в `result.md`.

### 2. CLI
Интерфейс (clap или эквивалент):

```
harness.exe --help
harness.exe [--conf <path>] --prompt "<user prompt>"
harness.exe [--conf <path>] --dump "<path to dump file>"
```

Правила:
- `--help` — справка по всем флагам и завершение с кодом 0.
- `--conf` — опционален. По умолчанию: `<process-cwd>/harness/harness.json5` (cwd запуска процесса, не `settings.cwd`).
- `--prompt` — стартовый запрос новой сессии. Поле `prompt` в json5 **игнорировать** (источник промпта — только CLI).
- `--dump` — продолжение сессии из dump-файла. Промпт из dump-истории, `--prompt` не используется.
- `--prompt` и `--dump` взаимоисключающие. Заданы оба → ошибка CLI и ненулевой код.
- Не задан ни один → ошибка CLI («нужен --prompt или --dump») и ненулевой код.
- Бинарник не должен требовать `invoke` для запуска.

### 3. Конфигурация
Отдельный модуль (например `config`) отвечает за:
- поиск файла (`--conf` / default);
- разбор JSON5;
- загрузку вложенного файла модели;
- дефолты и валидацию;
- выдачу **типизированных** структур (не `HashMap`/`serde_json::Value` как основной контракт наружу).

Схема `harness/harness.json5` (как есть):

```json5
{
    "cwd": "<workspace агента>",
    "paths": { "log-folder": ".log" },
    "agent-setting": {
        "config": "grok-4.6.json5",   // путь относительно директории файла --conf
        "mode": "safe"                // "safe" | "auto-accept"
    },
    "tool-settings": {
        "shell": [ "r:<cmd>", "w:<cmd>" ],
        "dirs":  [ "r:<dir>", "w:<dir>" ],
        "files": [ "r:<file>", "w:<file>" ],
        "available-file-extensions": [ /* пусто = все расширения */ ]
    }
}
```

Файл модели (`agent-setting.config`), эталон `harness/grok-4.6.json5`:

```json5
{
    "name": "Grok 4.6",
    "iteration-limit": 50,
    "token-limit": -1,     // -1 = без лимита
    "USD-limit": 0.5       // -1 = без лимита
}
```

Правила загрузки:
- `cwd` в конфиге — workspace агента (tools, логи, invoke). Не путать с process-cwd.
- Путь модели резолвить относительно директории файла `--conf`.
- Vendor **не** хранить отдельным полем: выводить из `name` модели так же, как Python `AgentLoop` + `from_str`:
  - `"Grok 4.6"` → SpaceXAI / xAI, api model `grok-4.6`;
  - `"Gemini 3.8 Flash"` → GoogleAI, `gemini-3.8-flash`;
  - `"Gemini 3.5 Flash Lite"` → `gemini-3.5-flash-lite`;
  - `"Gemini 3.1 Flash Lite"` → `gemini-3.1-flash-lite`;
  - неизвестное имя → ошибка старта.
- `mode`: `safe` → подтверждения лимитов включены; `auto-accept` (алиас `auto accept`) → без подтверждений. Иное значение → ошибка старта.
- Дефолты, которые Python подставляет в runtime и которых может не быть в json5:
  - `tool-settings.cwd` = `settings.cwd`;
  - `tool-settings.temp-dir` = каталог текущей log-сессии;
  - `command-execution-limit` = 120 с;
  - `invoke-execution-limit` = 120 с.
- Дубли ключей JSON5: последнее значение побеждает (в эталоне `grok-4.6.json5` `token-limit` записан дважды).

Hardcoded spec моделей (как в Python, с override из файла модели):

| name | api model | tls default | usd default | rpm | tpm | rpd |
|---|---|---|---|---|---|---|
| Grok 4.6 | grok-4.6 | 500000 | -1 | — | — | — |
| Gemini 3.8 Flash | gemini-3.8-flash | -1 | -1 (не используется) | 5 | 250000 | 20 |
| Gemini 3.5 Flash Lite | gemini-3.5-flash-lite | -1 | -1 | 15 | 250000 | 500 |
| Gemini 3.1 Flash Lite | gemini-3.1-flash-lite | -1 | -1 | 15 | 250000 | 500 |

Файл модели перекрывает `token-limit` / `USD-limit` / `iteration-limit`. Для Google `limit_usd` в runtime всегда `-1`, `consumed_usd` всегда `0.0` (как `GoogleAI` в Python).

### 4. Базовая архитектура (сохранить)
Слои те же, что в Python, в idiomatic Rust (traits + composition, не классы ради классов):

```
CLI → config → AgentLoop → Agent (xAI | Google)
                           ↘ Logger
                           ↘ ToolRegistry (категории tools)
```

Контракты:

**Logger** (`ILogger` / `DoubleLogger`):
- `log_line(message)` / `log_str(message)` дублируют вывод в stdout и в `log.md`.
- `log_line` гарантирует ведущий `\n`, если его нет.
- Цепочка вызовов может быть в Rust не fluent — поведение записи сохранить.

**Agent** (trait, эталон `IAgent`):
- `vendor`, `model`, `tokens_limit`, `consumed_tokens`, `limit_usd`, `consumed_usd`;
- `iteration(prompt: Option<&str>) -> Result<bool>`: `true` = сессия завершена (нет tool calls), `false` = нужна ещё итерация;
- `finish()` — статистика в лог;
- `dump_session(path)`, `reload_session(path)`.

**AgentLoop** (эталон `tools/harness/agent_loop.py`):
- Создаёт logger, tools, агента по vendor.
- Каталог сессии: `{cwd}/{paths.log-folder}/{YYYY-MM-DD HH-MM}_{vendor}_{model}/` (`mkdir -p`), внутри `log.md` и `dump.json`.
- При `--dump`: `reload_session(source_dump)`, стартовый prompt = `None`. Новый log-каталог всё равно создаётся; новый `dump.json` пишется туда (как в Python).
- Шапка лога: agent, iteration/token/usd limits, start time, опционально `continue from dump`.
- Цикл: `iteration += 1` → заголовок итерации с consumed tokens/usd → проверки лимитов → `agent.iteration(prompt)` → `dump_session` → если stop: `finish` и выход, иначе `prompt = None`.
- Проверка лимита: `limit != -1 && consumed > limit`. Сообщение `⚠️ ... exceed (...)`. В `safe` — `resume execution? [y/N]`, продолжать только при `y`/`yes`. В `auto-accept` — только лог, цикл продолжается. Отказ → прервать loop.
- Iteration limit срабатывает на итерации `limit+1` (сравнение `consumed > limit` после инкремента).
- В конце: `# Execution time: H:MM:SS`.

**API-ключи** (как в Python, relative к process-cwd):
- xAI: файл `API_KEY_XAI`;
- Google: файл `API_KEY_GEMINI`.

Официальных Rust SDK нет — HTTP (`reqwest` или аналог):
- xAI: OpenAI-compatible `https://api.x.ai/v1/chat/completions`, header `x-grok-conv-id`, tools = function calling. Счётчики: `consumed_tokens = input + output - cached`; `consumed_usd` из ответа, если есть.
- Google: `generateContent` REST; `automatic_function_calling` выключен. Rate limiter по `rpm`. Retry до 24 попыток: 429 — лог `fail 429: model limit exceeded!` и повтор; 503 — sleep random 5..=20 с и лог; иное — ошибка. `consumed_tokens = input + output`.

Dump — внутренний JSON агента (usage + history + pending tool results). Совместимость с Python-dump **не** требуется; свой dump должен успешно `reload`.

Логирование итерации агента сохранить по смыслу: user prompt / reasoning / agent content / tool calls (success/fail) в markdown-блоках.

### 5. Tools: подключаемые категории
Не переносить монолит `DefaultTools`. Каждая категория — отдельный модуль, регистрируемый в registry.

Registry:
- собирает descriptors всех категорий (`name`, `description`, JSON Schema parameters) — это `tools.list` для агента;
- `call(name, args) -> String` диспатчит в категорию;
- неизвестное имя → ошибка `tool '<name>' not available`;
- добавление категории не должно требовать правок `AgentLoop` / агентов (только точка сборки registry).

Общий sandbox (эталон `_check_access` / `_check_extensions` / `_is_allowed`):
- `w:` включает `r:`;
- директории — по префиксу `abspath`;
- файлы — точное совпадение `abspath`;
- пустой `available-file-extensions` = любые расширения;
- `request_*_access_for` мутирует allow-list в runtime → sandbox за `Mutex`/`RefCell`.

Категории и состав (имена, schema, тексты ошибок, коды возврата — 1:1 с Python + `tool_help.md`):

1. **fs** — `read_file`, `write_file`, `create_file`, `remove_file`, `move_file`, `list_folder`, `create_folder`, `remove_folder`, `move_folder`.
2. **git** — `is_file_under_git`, `get_file_diff`, `apply_diff_patch`, `discard_file_changes` (через бинарь `git`, не `git2`).
3. **search** — `find_str`, `find_str_by_regex` (прогресс-сводка в stdout с перезаписью предыдущей, формат как в Python).
4. **shell** — `run_shell`, `run_invoke`.
5. **meta** — `list_available_file_extension`, `list_available_r_dir`, `list_available_w_dir`, `list_available_r_file`, `list_available_w_file`, `list_available_shell_commands`.
6. **user** — `ask_user`, `request_read_access_for`, `request_write_access_for`, `request_command_shell_execution`.

Ключевые инварианты эталона (не «улучшать», если не сломано):
- `write_file` создаёт parent dirs; `create_file` — нет, существующий файл очищает.
- `list_folder`: `depth` приходит строкой; `0` — пустой результат; сначала dirs (рекурсия `depth-1`), затем files; имена сортированы.
- `apply_diff_patch`: цели из заголовков unified diff; все цели требуют `w`; патч во временный файл в `temp-dir` с завершающим `\n`; `git apply --ignore-whitespace --ignore-space-change --recount` из `settings.cwd`.
- git-команды (`ls-files` / `diff HEAD` / `checkout HEAD`) выполняются с `cwd = settings.cwd` и путём как пришёл в tool.
- `run_shell`: whitelist; `w:`-команда требует `w` на `cwd`, иначе `r`; `cmd = command + " " + arguments`; `arguments` в schema не required — отсутствие трактовать как `""`; вывод стримить в stdout сразу; timeout `command-execution-limit`; decode `utf-8` → `oem` → `cp1251`; пустой вывод → `(command finished without output)`.
- `run_invoke`: **не** через whitelist. Команда:
  - Windows: `{cwd}/.venv/Scripts/python.exe -m invoke {command}`;
  - Unix: `{cwd}/.venv/bin/python -m invoke {command}`;
  - cwd процесса invoke = `settings.cwd`; timeout `invoke-execution-limit`.
- `request_command_shell_execution` при согласии запускает команду один раз, **не** добавляя в whitelist.
- Согласие доступа: `y`/`yes`/`д`/`да`. Существующая директория → prefix-list, иначе file-list; `w` добавляет и в `r`.
- После `call` логировать результат так же, как `DefaultTools.call` (особые случаи `apply_diff_patch` / `read_file` / `write_file`).
- Короткие description tools — как в `default_tools.py` (агенту уходит schema+description, не `tool_help.md`). `verbosity_help` в CLI-tools не экспортировать.

### 6. Код и зависимости
- Idiomatic Rust: traits, статический dispatch где возможно; `dyn` — для гетерогенного списка категорий.
- Комментарий-шапка у каждого модуля; rustdoc на публичных типах/функциях/трейтах.
- `Cargo.toml`: короткий комментарий над каждой зависимостью, зачем она; висячие не оставлять.
- Секреты и лимиты не хардкодить — только конфиг / файлы ключей / CLI.
- Рефакторинг после рабочего минимума: `WiKi/harness/rules/refactoring-base.md` + `WiKi/harness/rules/refactoring-rust.md`.
- Этап-1: прямой рабочий код + unit-тесты на нетривиальную логику. Этап-2: рефакторинг. Оба этапа в одном прогоне задачи.

Минимальные тесты (без живых LLM-вызовов):
- парсинг json5 + вложенный model-файл + дефолты;
- CLI: xor `--prompt`/`--dump`, default `--conf`, `--help`;
- sandbox: prefix dir / exact file / w⊃r / extensions;
- registry: dispatch по имени, unknown tool;
- git patch path extraction;
- взаимное исключение CLI-аргументов.

## Scope
Создать:
- `tools/harness-rs/Cargo.toml`
- `tools/harness-rs/src/**` (CLI, config, logger, agent_loop, agents, tools-категории, tests)

Читать как эталон, не менять:
- `tools/harness/__init__.py`
- `tools/harness/agent_loop.py`
- `tools/harness/logger.py`
- `tools/harness/agents/**`
- `tools/harness/agents/tools/default_tools.py`
- `tools/harness/agents/tools/i_tools.py`
- `tools/harness/agents/tools/tool_help.md`
- `harness/harness.json5`
- `harness/grok-4.6.json5`
- `WiKi/harness/rules/refactoring-base.md`
- `WiKi/harness/rules/refactoring-rust.md`

Менять `harness/*.json5` только при нехватке обязательного поля.

Не входит:
- удаление Python-harness;
- совместимость dump с Python;
- новые tools / новые vendor;
- GUI / TUI.

## Проверка корректности решения
1. `cargo test` в `tools/harness-rs` — зелёный.
2. `cargo run -- --help` — справка, код 0.
3. Без `--prompt`/`--dump` — ненулевой код и понятная ошибка.
4. `--prompt` вместе с `--dump` — ненулевой код.
5. Ручной шаг (если есть ключ):  
   `harness.exe --conf harness/harness.json5 --prompt "ping"`  
   → создан `.log/<date>_<vendor>_<model>/` с `log.md` и `dump.json`; в логе шапка, итерации, finish/stats.  
   Повтор с `--dump <тот dump.json>` — в логе `continue from dump`, история восстановлена.
6. Живой вызов LLM для merge не обязателен, если пункты 1–4 зелёные и модули агента покрыты разбором ответа/tool-call на фикстурах. Если живой вызов делается — не жечь лимиты: один короткий prompt.

# Дополнительные указания
1. Экономь токены: читай минимум необходимого; правки вноси через patch.
2. Если что-то не получается со второго раза, или ты понимаешь, что контекст задачи сильно разрастается относительно цели задачи в минимальном воплощении, или что-то какая-то информация не дана, но является важной для правильной реализации поставленной задачи → не фантазируй и не додумывай за меня → остановись и задай вопрос → я подскажу и направлю.
3. Результаты — в `result.md` рядом с `todo.md`, формат `было→стало(почему)`.
4. Действия с моей стороны — в `note.md` рядом с `todo.md`.
