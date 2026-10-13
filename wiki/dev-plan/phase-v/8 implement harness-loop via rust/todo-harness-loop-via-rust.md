# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача на разработку: перенос harness-loop с Python на чистый Rust
Реализовать нативный Rust-бинарник `harness.exe`, эквивалентный текущему Python-harness (`tools/harness`) по поведению agent-loop, с сохранением базовой архитектуры и целевыми изменениями:

1. Cargo workspace: бинарник + отдельные крейты `conf`, `logger` и `tools`.
2. Tools — отдельные модули (`один tool-набор = один модуль = одна запись в конфиге`), каждый со своими настройками и unit-тестами в конце файла.
3. Отдельный крейт `conf` (`harness/crates/conf`) — единственный, кто читает файлы конфигурации (`harness.json5` и вложенный model-файл); основной файл — `./harness/harness.json5`. Все ключи json5 и default-значения — только в `harness/crates/conf/src/str/keys.rs`.
4. Путь до исполняемого файла `git` — только из конфига, не хардкод.
5. API-ключ модели — только из обязательного поля `api-key` файла модели (путь к файлу с ключом **или** сам ключ строкой). Хардкод имён `API_KEY_XAI` / `API_KEY_GEMINI` запрещён.
6. Запуск только через CLI `harness.exe` (не через `invoke`).
7. `harness.exe --conf-new <path>` создаёт новый файл конфигурации в максимально полном наполнении (все модули, комментарии, дефолты, заглушки для ручных полей).

Python-код (`tools/harness/**`) — эталон поведения. Его не менять и не удалять.

Зачем: уйти от Python/invoke-обвязки к одному нативному бинарнику; вынести чтение конфигурации в отдельный крейт с единой таблицей ключей; разнести tools по модулям с явной записью в конфиге; вынести логирование в отдельный крейт; сделать конфигурацию и CLI явным слоем, а не словарями внутри `invoke.task`; не хранить API-ключи и пути к ним в коде агентов.

Ожидаемый результат:
- Cargo workspace в `./harness`, собирающий `harness.exe`.
- Крейт `harness/crates/conf`: загрузка/валидация json5, типизированные настройки, таблица ключей/дефолтов, генерация шаблона `--conf-new`. Никакой другой крейт не открывает `harness.json5` / model-файлы.
- Крейт `harness/crates/logger` с интерфейсом логирования, который передаётся в loop и во все tools.
- Крейт `harness/crates/tools` с модулем на каждый tool-набор и тестами в конце каждого модуля.
- `harness.exe --help` печатает справку (включая `--conf-new`).
- `harness.exe --prompt "..."` стартует новую сессию по конфигу.
- `harness.exe --dump <path>` продолжает сессию из dump-файла.
- `harness.exe --conf-new <path>` пишет полный шаблон json5 и завершается, loop не стартует.
- Поведение loop / sandbox / tools / лимитов / логов совпадает с Python-эталоном (кроме формата dump: совместимость с Python-dump не требуется; кроме новой схемы `tools` в json5, пути к git из конфига и `api-key` из файла модели).

## Функциональные требования

### 1. Расположение и границы
Cargo workspace `./harness`:

```
harness/
  Cargo.toml                          # [workspace], members = crates/*
  harness.json5                       # основной conf (схема — см. §3)
  grok-4.6.json5                      # файл модели (эталон формата, уже содержит api-key)
  crates/
    harness/                          # bin crate, package/bin name: harness
      Cargo.toml                      # зависит от conf, logger, tools
      src/main.rs                     # CLI entry
      src/cli.rs
      src/agent_loop.rs
      src/agents/                     # xAI / Google
    conf/
      Cargo.toml                      # package name: conf
      src/lib.rs                      # загрузка/валидация json5, типы настроек, генерация шаблона
      src/str/keys.rs                 # все ключи json5 и значения по умолчанию
    logger/
      Cargo.toml                      # package name: logger
      src/lib.rs                      # trait Logger + реализации (реализации в отдельных файлах)
    tools/
      Cargo.toml                      # package name: tools, зависит от logger и conf (типы настроек)
      src/lib.rs                      # registry + сборка модулей из конфига
      src/sandbox.rs                  # общий access-control (не tool, без записи в conf)
      src/fs.rs
      src/git.rs
      src/search.rs
      src/terminal.rs
      src/meta.rs
      src/user.rs
```

- Эталон поведения (read-only): `tools/harness/**`.
- Эталон формата модели (read-only, кроме создания недостающих model-файлов): `harness/grok-4.6.json5`.
- Python не модифицировать.
- `harness/harness.json5` **обязательно** мигрировать со схемы `tool-settings` на схему `tools` (§3). Фактические allow-list пути текущего файла сохранить, только переложить в новые ключи. Прочие `harness/*.json5` менять только если текущей схемы модели не хватает для обязательных полей; любое расширение — минимальное, описать в `result.md`.
- Единственный крейт, которому разрешено читать файлы конфигурации (`harness.json5` и вложенный model json5) — `conf`. Бинарник, `logger`, `tools`, агенты получают уже разобранные типизированные настройки из `conf` и сами json5 не парсят.

### 2. CLI
Интерфейс (clap или эквивалент):

```
harness.exe --help
harness.exe [--conf <path>] --prompt "<user prompt>"
harness.exe [--conf <path>] --dump "<path to dump file>"
harness.exe --conf-new "<path to new conf file>"
```

Правила:
- `--help` — справка по всем флагам (включая `--conf-new`) и завершение с кодом 0.
- `--conf` — опционален. По умолчанию: `<process-cwd>/harness.json5` (cwd запуска процесса, не `settings.cwd`). Не используется вместе с `--conf-new`.
- `--prompt` — стартовый запрос новой сессии.
- `--dump` — продолжение сессии из dump-файла. Промпт из dump-истории, `--prompt` не используется.
- `--conf-new` — создать новый файл конфигурации по пути и выйти (loop / агент / сеть не трогать).
- `--prompt`, `--dump` и `--conf-new` взаимоисключающие. Заданы двое или трое → ошибка CLI и ненулевой код.
- Не задан ни один из `--prompt` / `--dump` / `--conf-new` → ошибка CLI («нужен --prompt, --dump или --conf-new») и ненулевой код.
- `--conf` вместе с `--conf-new` → ошибка CLI и ненулевой код.
- Бинарник не должен требовать `invoke` для запуска.

`--conf-new <path>`:
- Генерация живёт в крейте `conf` (функция записи шаблона); CLI только передаёт путь.
- Писать **текстовый JSON5-шаблон с комментариями**, не round-trip через serde (комментарии иначе потеряются).
- Максимально полное наполнение:
  - все известные tool-модули в объекте `tools` (включённые, с полным набором полей);
  - комментарий-подпись у каждого поля / блока (смысл ключа, ограничения, примеры);
  - значения по умолчанию, если они есть в `src/str/keys.rs`;
  - заглушки для полей, которые пользователь обязан задать вручную (`cwd`, `tools.git.executable`, `agent-setting.config`, пути allow-list и т.п.) — явные placeholder-строки, не пустые «угадайки».
- Если файл по `path` уже существует → ошибка, не перезаписывать.
- Если родительской директории нет → создать (`mkdir -p`) и записать файл.
- Успех → код 0. Шаблон model-файла эта команда не создаёт.

### 3. Конфигурация (крейт `harness/crates/conf`)
Крейт `conf` — единственная точка чтения конфигурации. Остальные крейты используют его публичный API, чтобы получить типизированные настройки.

Отвечает за:
- поиск файла (`--conf` / default) — по запросу бинарника;
- разбор JSON5 основного файла и вложенного файла модели;
- дефолты и валидацию;
- выдачу **типизированных** структур (не `HashMap`/`serde_json::Value` как основной контракт наружу);
- генерацию шаблона `--conf-new`.

Зависимости:
- `conf` не зависит от `logger`, `tools`, бинарника;
- `tools` зависит от `conf` только ради типов настроек tool-модулей (сами json5-файлы не открывает);
- бинарник зависит от `conf` и передаёт настройки в loop / агентов / tools.

#### 3.1. Ключи и дефолты — `src/str/keys.rs`
Все строковые ключи, по которым читаются значения из `harness.json5` и model-файла, а также все значения по умолчанию (если они есть), живут **только** в модуле `harness/crates/conf/src/str/keys.rs`.

Правила:
- в `lib.rs` и прочих файлах крейта запрещены «голые» литералы ключей вроде `"cwd"`, `"api-key"`, `"tools"` — только константы из `str::keys`;
- default-значения (например `log-folder`, лимиты-заглушки шаблона) тоже только оттуда;
- смена имени ключа в json5 = правка одного модуля.

#### 3.2. Схема `harness/harness.json5`

Целевая схема `harness/harness.json5`:

```json5
{
    "cwd": "<workspace агента>",      // должен быть явно задан, иначе ошибка
    "paths": { "log-folder": ".log" },
    "agent-setting": {
        "config": "grok-4.6.json5",   // путь относительно директории файла --conf
        "mode": "safe"                // "safe" | "auto-accept"
    },
    // ключ объекта = имя tool-модуля (harness/crates/tools/src/<name>.rs).
    // нет ключа → модуль не регистрируется, его tools агенту недоступны.
    // неизвестный ключ → ошибка старта.
    "tools": {
        "fs": {
            "available-dirs": [
                "w:tools/harness/agents/tools",
                "r:WiKi/harness/rules",
                "r:WiKi/harness/roles",
            ],
            "available-file-extensions": [
                // пустой список = любые расширения
            ],
            "extra-scope": [
                // точечный доступ к файлам вне available-dirs; префиксы r: / w:
            ]
        },
        "git": {
            // обязателен. Код не подставляет "git" сам.
            "executable": "path to git-executable file",
            "execution-limit": 120
        },
        "search": {},
        "terminal": {
            "cwd-list": [
                // директории, из которых можно выполнять терминальные команды
            ],
            "available-commands": [
                "findstr",
                "cargo",
            ],
            "execution-limit": 120,
        },
        "invoke": {
          "cwd": "path to tasks.py for invoke",
          "execution-limit": 120
        },
        "meta": {},
        "user": {}
    }
}
```

Файл модели (`agent-setting.config`), эталон `harness/grok-4.6.json5`:

```json5
{
    "vendor": "SpaceXAI",
    "name": "Grok 4.6",
    "api-key": "api-key or path to file with api-key",
    "token-limit": 128000,
    "iteration-limit": 50,
    "USD-limit": 0.5
}
```

Правила загрузки:
- `cwd` в конфиге — workspace агента (tools, логи, invoke). Не путать с process-cwd.
- Путь модели резолвить относительно директории файла `--conf`.
- `vendor` + `name` определяют целевого агента для harness-loop.
  - если целевая модель требует лимиты, которые не заданы в конфигурации, выдавать ошибку с просьбой определить целевые лимиты в conf-файле (лимиты и ограничения не должны храниться в коде).
  - неизвестный агент → ошибка старта.
- `api-key` в файле модели **обязателен** (см. §3.3). Отсутствие ключа или пустая строка → ошибка старта с просьбой задать `api-key` в model-файле.
- `mode`: `safe` → подтверждения лимитов включены; `auto-accept` → без подтверждений. Иное значение → ошибка старта.
- Runtime-подстановки (как Python, но без словаря `tool-settings`):
  - workspace tools = `settings.cwd`;
  - `temp-dir` = каталог текущей log-сессии (передаётся в tools-контекст при старте loop, в json5 не хранится).
- Лимиты исполнения команд — только из conf, без дефолтов в коде:
  - `tools.terminal.command-execution-limit`, `tools.terminal.invoke-execution-limit` обязательны, если модуль `terminal` включён;
  - `tools.git.execution-limit` и `tools.git.executable` обязательны, если модуль `git` включён.
- Объект `tools`:
  - порядок ключей не важен;
  - значение каждого ключа — объект настроек этого модуля (пустой объект `{}` допустим, если модулю нечего конфигурировать);
  - отсутствие ключа = модуль выключен;
  - неизвестное имя модуля → ошибка старта.

Для известных моделей создать файлы конфигурации с лимитами в `./harness/<model-name>.json5` (как в Python, с override из файла модели). Каждый файл **обязан** содержать `api-key` (заглушка пути/ключа, как в `harness/grok-4.6.json5`; реальный секрет в репозиторий не класть):

| name | api model | tls default | usd default | rpm | tpm | rpd |
|---|---|---|---|---|---|---|
| Grok 4.6 | grok-4.6 | 500000 | -1 | — | — | — |
| Gemini 3.8 Flash | gemini-3.8-flash | -1 | -1 (не используется) | 5 | 250000 | 20 |
| Gemini 3.5 Flash Lite | gemini-3.5-flash-lite | -1 | -1 | 15 | 250000 | 500 |
| Gemini 3.1 Flash Lite | gemini-3.1-flash-lite | -1 | -1 | 15 | 250000 | 500 |

Файл модели перекрывает `token-limit` / `USD-limit` / `iteration-limit`. Для Google `limit_usd` в runtime всегда `-1`, `consumed_usd` всегда `0.0` (как `GoogleAI` в Python).

Миграция текущего `harness/harness.json5`:
- блок `tool-settings` удалить;
- `tool-settings.dirs` → `tools.fs.available-dirs`;
- `tool-settings.files` → `tools.fs.extra-scope`;
- `tool-settings.available-file-extensions` → `tools.fs.available-file-extensions`;
- `tool-settings.shell` (`r:cmd` / `w:cmd`) → `tools.terminal.available-commands` (имена **без** префиксов `r:`/`w:`);
- `tools.terminal.cwd-list` заполнить осознанно (в Python cwd команд проверялся через `dirs`; теперь это отдельный список);
- добавить `tools.git.executable` (обязательный путь/имя бинаря git);
- включить остальные модули (`search`, `meta`, `user`) пустыми объектами, если они нужны для паритета с Python-эталоном.

#### 3.3. `api-key` в файле модели
Во **всех** файлах конфигурации моделей обязателен ключ `api-key`: путь к файлу с ключом **либо** сам ключ строкой.

Правила:
- нет ключа / значение не строка / пустая строка после trim → ошибка старта;
- если значение указывает на **существующий файл** (абсолютный путь или относительно process-cwd) — прочитать содержимое файла как UTF-8, trim whitespace по краям, это и есть секрет;
- иначе значение используется как ключ целиком;
- агенты получают уже разрешённую строку ключа из настроек `conf`, сами файлы ключей и json5 не открывают;
- имена `API_KEY_XAI` / `API_KEY_GEMINI` в коде не фигурируют.

### 4. Базовая архитектура (сохранить)
Слои те же, что в Python, в idiomatic Rust (traits + composition, не классы ради классов):

```
CLI → conf (crate) → AgentLoop → Agent (xAI | Google)
                                 ↘ Logger (crate logger, Arc<dyn Logger>)
                                 ↘ ToolRegistry (crate tools: fs | git | search | terminal | meta | user)
```

Контракты:

**Logger** — см. §5 (крейт `logger`). В loop и во все tool-модули передаётся один и тот же экземпляр (`Arc<dyn Logger>`).

**Agent** (trait, эталон `IAgent`):
- `vendor`, `model`, `tokens_limit`, `consumed_tokens`, `limit_usd`, `consumed_usd`;
- `iteration(prompt: Option<&str>) -> Result<bool>`: `true` = сессия завершена (нет tool calls), `false` = нужна ещё итерация;
- `finish()` — статистика в лог;
- `dump_session(path)`, `reload_session(path)`.

**AgentLoop** (эталон `tools/harness/agent_loop.py`):
- Создаёт logger, tools-registry, агента по vendor.
- Каталог сессии: `{cwd}/{paths.log-folder}/{YYYY-MM-DD HH-MM}_{vendor}_{model}/` (`mkdir -p`), внутри `log.md` и `dump.json`.
- При `--dump`: `reload_session(source_dump)`, стартовый prompt = `None`. Новый log-каталог всё равно создаётся; новый `dump.json` пишется туда (как в Python).
- Шапка лога: agent, iteration/token/usd limits, start time, опционально `continue from dump`.
- Цикл: `iteration += 1` → заголовок итерации с consumed tokens/usd → проверки лимитов → `agent.iteration(prompt)` → `dump_session` → если stop: `finish` и выход, иначе `prompt = None`.
- Проверка лимита: `limit != -1 && consumed > limit`. Сообщение `⚠️ ... exceed (...)`. В `safe` — `resume execution? [y/N]`, продолжать только при `y`/`yes`. В `auto-accept` — только лог, цикл продолжается. Отказ → прервать loop.
- Iteration limit срабатывает на итерации `limit+1` (сравнение `consumed > limit` после инкремента).
- В конце: `# Execution time: H:MM:SS`.

**API-ключ** — из `api-key` файла модели через крейт `conf` (§3.3). В конструктор агента передаётся уже прочитанная строка, не путь и не имя vendor-файла.

Официальных Rust SDK нет — HTTP (`reqwest` или аналог):
- xAI: OpenAI-compatible `https://api.x.ai/v1/chat/completions`, header `x-grok-conv-id`, tools = function calling. Счётчики: `consumed_tokens = input + output - cached`; `consumed_usd` из ответа, если есть.
- Google: `generateContent` REST; `automatic_function_calling` выключен. Rate limiter по `rpm`. Retry до 24 попыток: 429 — лог `fail 429: model limit exceeded!` и повтор; 503 — sleep random 5..=20 с и лог; иное — ошибка. `consumed_tokens = input + output`.

Dump — внутренний JSON агента (usage + history + pending tool results). Совместимость с Python-dump **не** требуется; свой dump должен успешно `reload`.

Логирование итерации агента сохранить по смыслу: user prompt / reasoning / agent content / tool calls (success/fail) в markdown-блоках.

### 5. Крейт логирования (`harness/crates/logger`)
Логирование — отдельный крейт. Loop, агенты и каждый tool-модуль получают логгер снаружи, сами файлы/`println!` для журнала сессии не открывают.

Интерфейс (адекватный стартовый контракт; при необходимости будет поправлен отдельно):

```rust
/// Потокобезопасный журнал сессии harness.
///
/// Реализации дублируют запись туда, куда нужно вызывающему коду
/// (stdout+файл, только память в тестах и т.д.).
pub trait Logger: Send + Sync {
    /// Записать фрагмент как есть, без принудительного перевода строки.
    fn log_str(&self, message: &str);

    /// Записать строку. Если `message` не начинается с `\n`, префикс `\n` добавляется.
    fn log_line(&self, message: &str);
}
```

Обязательные реализации в том же крейте:
- `DoubleLogger` — эталон Python `DoubleLogger`: создаёт/touch файл, каждая запись идёт в stdout **и** в файл (`log.md`). Конструктор принимает путь к файлу лога.
- `NullLogger` — no-op для unit-тестов tools.

Правила:
- Цепочка вызовов в Rust **не** fluent: методы возвращают `()`. Поведение записи — 1:1 с Python.
- В tools/loop логгер хранить как `Arc<dyn Logger>` (гетерогенная передача в модули).
- Исключения, которые **не** идут через `Logger` (как в Python):
  - стриминг stdout дочернего процесса (`run_shell` / `run_invoke`);
  - progress-сводка `find_str` / `find_str_by_regex` с перезаписью предыдущих строк.
- Крейт не знает про агентов и tools: только trait + реализации.

### 6. Tools: один набор — один модуль — одна запись в конфиге
Не переносить монолит `DefaultTools`.

Крейт `harness/crates/tools`:
- каждый tool-набор — файл `src/<name>.rs`;
- в **конце каждого** такого файла — `#[cfg(test)] mod tests` с минимальными unit-тестами, гарантирующими работоспособность набора;
- имя модуля = ключ в `settings.tools`;
- внутренние модули без ключа в conf допустимы только как инфраструктура (`sandbox`, хелперы запуска процессов). Они не экспортируют tools агенту.

Registry (`src/lib.rs`):
- по объекту `tools` из конфига создаёт и регистрирует только включённые модули;
- собирает descriptors всех модулей (`name`, `description`, JSON Schema parameters) — это `tools.list` для агента;
- `call(name, args) -> String` диспатчит в модуль-владелец;
- неизвестное имя → ошибка `tool '<name>' not available`;
- добавление набора: новый `src/<name>.rs` + ветка в точке сборки registry. `AgentLoop` / агенты не трогать.

Контекст, который loop передаёт в крейт tools при сборке:
- `cwd` агента, `temp-dir` сессии, `Arc<dyn Logger>`, `safe_mode` не нужен tools (подтверждения лимитов — в loop; `request_*` всегда спрашивают пользователя, как в Python).

**Sandbox** (`src/sandbox.rs`, общий, за `Mutex`):
- инициализируется из `tools.fs` (если ключа `fs` нет — пустые списки);
- `w:` включает `r:`;
- директории (`available-dirs`) — по префиксу `abspath`;
- файлы (`extra-scope`) — точное совпадение `abspath`;
- пустой `available-file-extensions` = любые расширения;
- `request_*_access_for` мутирует allow-list в runtime;
- git / search / fs / user работают с **одним** sandbox.

Категории (имена tools, schema, тексты ошибок, коды возврата — 1:1 с Python + `tool_help.md`):

1. **fs** (`fs.rs`, ключ `tools.fs`) — `read_file`, `write_file`, `create_file`, `remove_file`, `move_file`, `list_folder`, `create_folder`, `remove_folder`, `move_folder`.
   - настройки: `available-dirs`, `available-file-extensions`, `extra-scope`.
2. **git** (`git.rs`, ключ `tools.git`) — `is_file_under_git`, `get_file_diff`, `apply_diff_patch`, `discard_file_changes`.
   - через бинарь из `tools.git.executable` (`Command::new(executable)`), **не** `git2`, **не** хардкод `"git"`.
   - timeout = `tools.git.execution-limit`.
   - функционал 1:1 с Python, новое не придумывать.
3. **search** (`search.rs`, ключ `tools.search`) — `find_str`, `find_str_by_regex` (прогресс-сводка в stdout с перезаписью предыдущей, формат как в Python). Доступ к файлам — через общий sandbox.
4. **terminal** (`terminal.rs`, ключ `tools.terminal`) — `run_shell`, `run_invoke`.
   - `available-commands` — имена команд без префиксов `r:`/`w:`.
   - `cwd-list` — разрешённые cwd для `run_shell` (проверка по префиксу `abspath`, отдельно от `fs.available-dirs`).
   - `run_shell`: команда должна быть в `available-commands`, cwd — в `cwd-list`; `cmd = command + " " + arguments`; `arguments` в schema не required — отсутствие трактовать как `""`.
5. **meta** (`meta.rs`, ключ `tools.meta`) — `list_available_file_extension`, `list_available_r_dir`, `list_available_w_dir`, `list_available_r_file`, `list_available_w_file`, `list_available_shell_commands`. Читает текущее состояние sandbox и terminal-whitelist.
6. **user** (`user.rs`, ключ `tools.user`) — `ask_user`, `request_read_access_for`, `request_write_access_for`, `request_command_shell_execution`.

Ключевые инварианты эталона (не «улучшать», если не сломано):
- `write_file` создаёт parent dirs; `create_file` — нет, существующий файл очищает.
- `list_folder`: `depth` приходит строкой; `0` — пустой результат; сначала dirs (рекурсия `depth-1`), затем files; имена сортированы.
- `apply_diff_patch`: цели из заголовков unified diff; все цели требуют `w`; патч во временный файл в `temp-dir` с завершающим `\n`; `git apply --ignore-whitespace --ignore-space-change --recount` из `settings.cwd`, программа = `tools.git.executable`.
- git-команды (`ls-files` / `diff HEAD` / `checkout HEAD`) выполняются с `cwd = settings.cwd` и путём как пришёл в tool; argv = `[executable, ...]`.
- `run_shell`: вывод стримить в stdout сразу; timeout `command-execution-limit`; decode `utf-8` → `oem` → `cp1251`; пустой вывод → `(command finished without output)`.
- `run_invoke`: **не** через whitelist команд. Команда:
  - Windows: `{cwd}/.venv/Scripts/python.exe -m invoke {command}`;
  - Unix: `{cwd}/.venv/bin/python -m invoke {command}`;
  - cwd процесса invoke = `settings.cwd`; timeout `invoke-execution-limit`.
- `request_command_shell_execution` при согласии запускает команду один раз, **не** добавляя в `available-commands`.
- Согласие доступа: `y`/`yes`/`д`/`да`. Существующая директория → prefix-list sandbox, иначе extra-scope; `w` добавляет и в `r`.
- После `call` логировать результат так же, как `DefaultTools.call` (особые случаи `apply_diff_patch` / `read_file` / `write_file`).
- Короткие description tools — как в `default_tools.py` (агенту уходит schema+description, не `tool_help.md`). `verbosity_help` в CLI-tools не экспортировать.

### 7. Код и зависимости
- Idiomatic Rust: traits, статический dispatch где возможно; `dyn` — для гетерогенного списка tool-модулей и для `Logger`.
- Комментарий-шапка у каждого модуля; rustdoc на публичных типах/функциях/трейтах.
- `Cargo.toml` (workspace и каждого крейта): короткий комментарий над каждой зависимостью, зачем она; висячие не оставлять.
- Секреты, лимиты и путь к git не хардкодить — только конфиг (`api-key` модели / `tools.git.executable` / лимиты json5) / CLI.
- Рефакторинг после рабочего минимума: `WiKi/harness/rules/refactoring-base.md` + `WiKi/harness/rules/refactoring-rust.md`.
- Этап-1: прямой рабочий код + unit-тесты на нетривиальную логику. Этап-2: рефакторинг. Оба этапа в одном прогоне задачи.

Минимальные тесты (без живых LLM-вызовов):
- парсинг json5 + вложенный model-файл + дефолты + объект `tools` (включён/выключен модуль, неизвестный ключ);
- ключи чтения json5 берутся из `conf::str::keys`, а не из литералов в тестах загрузчика (хотя бы smoke: переименование константы ломает разбор ожидаемым образом / константы совпадают с ключами схемы);
- CLI: xor `--prompt`/`--dump`/`--conf-new`, default `--conf`, `--help` содержит `--conf-new`;
- `--conf-new`: создаёт файл с комментариями и всеми tool-модулями; повтор по существующему пути — ошибка;
- `api-key`: отсутствует → ошибка; строка-ключ; путь к существующему файлу (читается содержимое);
- sandbox: prefix dir / exact file / w⊃r / extensions;
- registry: dispatch по имени, unknown tool, сборка только из ключей conf;
- git: path extraction патча; вызов идёт в `executable` из конфига (не в хардкод);
- взаимное исключение CLI-аргументов;
- unit-тесты **в конце каждого** tool-модуля.

## Scope
Создать:
- `harness/Cargo.toml` (workspace)
- `harness/crates/harness/**` (CLI, logger-wiring, agent_loop, agents)
- `harness/crates/conf/**` (загрузка json5, `src/str/keys.rs`, типы, `--conf-new`)
- `harness/crates/logger/**`
- `harness/crates/tools/**` (registry, sandbox, fs, git, search, terminal, meta, user + tests в конце модулей)

Обновить:
- `harness/harness.json5` — миграция `tool-settings` → `tools` (§3)

Читать как эталон, не менять:
- `tools/harness/__init__.py`
- `tools/harness/agent_loop.py`
- `tools/harness/logger.py`
- `tools/harness/agents/**`
- `tools/harness/agents/tools/default_tools.py`
- `tools/harness/agents/tools/i_tools.py`
- `tools/harness/agents/tools/tool_help.md`
- `harness/grok-4.6.json5` (кроме создания соседних model-файлов по таблице; `api-key` в эталоне уже есть — в новых model-файлах повторить)
- `WiKi/harness/rules/refactoring-base.md`
- `WiKi/harness/rules/refactoring-rust.md`

Не входит:
- удаление Python-harness;
- совместимость dump с Python;
- новые tools / новые vendor сверх перечисленных модулей;
- GUI / TUI;
- хардкод пути `git` / лимитов / имён файлов API-ключей в коде.

## Проверка корректности решения
1. `cargo test --workspace` в `./harness` — зелёный (включая тесты в конце каждого tool-модуля и тесты крейта `conf`).
2. `cargo run -p harness -- --help` — справка с `--conf-new`, код 0.
3. Без `--prompt`/`--dump`/`--conf-new` — ненулевой код и понятная ошибка.
4. `--prompt` вместе с `--dump` (и любая другая пара из `--prompt`/`--dump`/`--conf-new`) — ненулевой код.
5. Конфиг без `tools.git.executable` при включённом `git` — ошибка старта (не молчаливый fallback на `"git"`).
6. Model-файл без `api-key` — ошибка старта. Агенты не обращаются к `API_KEY_XAI` / `API_KEY_GEMINI`.
7. `harness.exe --conf-new <несуществующий-путь.json5>` — файл создан, внутри комментарии + все tool-модули + заглушки; код 0. Повтор с тем же путём — ошибка, файл не затёрт.
8. Ручной шаг (если есть ключ):
   `harness.exe --conf harness/harness.json5 --prompt "ping"`
   → создан `.log/<date>_<vendor>_<model>/` с `log.md` и `dump.json`; в логе шапка, итерации, finish/stats.
   Повтор с `--dump <тот dump.json>` — в логе `continue from dump`, история восстановлена.
9. Живой вызов LLM для merge не обязателен, если пункты 1–7 зелёные и модули агента покрыты разбором ответа/tool-call на фикстурах. Если живой вызов делается — не жечь лимиты: один короткий prompt.

# Дополнительные указания
1. Экономь токены: читай минимум необходимого; правки вноси через patch.
2. Если что-то не получается со второго раза, или ты понимаешь, что контекст задачи сильно разрастается относительно цели задачи в минимальном воплощении, или что-то какая-то информация не дана, но является важной для правильной реализации поставленной задачи → не фантазируй и не додумывай за меня → остановись и задай вопрос → я подскажу и направлю.
3. Результаты — в `result.md` рядом с `todo.md`, формат `было→стало(почему)`.
4. Действия с моей стороны — в `note.md` рядом с `todo.md`.
