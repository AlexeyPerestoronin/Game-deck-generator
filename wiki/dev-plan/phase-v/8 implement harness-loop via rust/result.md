# Результат: harness-loop на Rust

## Было → стало (почему)

- **Не было Cargo workspace** → `harness/Cargo.toml` + крейты `conf`, `logger`, `tools`, `harness` (бинарник `harness.exe`). Почему: один нативный запуск без Python/invoke.
- **Конфиг читался в Python `invoke.task` словарями** → крейт `conf` — единственный, кто открывает `harness.json5` и model-файл; ключи/дефолты только в `crates/conf/src/str/keys.rs`. Почему: смена имени ключа = правка одного модуля; агенты/tools json5 не парсят.
- **`harness/harness.json5`: блок `tool-settings`** → схема `tools` (`fs` / `git` / `search` / `terminal` / `meta` / `user`). Allow-list путей сохранён: `dirs`→`available-dirs`, `files`→`extra-scope`, `shell: r:findstr`→`available-commands: ["findstr"]` без префикса. Почему: один tool-набор = один модуль = одна запись в conf.
- **Лимиты сессии не были в `agent-setting`** → `token-limit` / `iteration-limit` / `USD-limit` обязательны в `agent-setting` (взяты из рекомендованных значений grok-файла: 128000 / 50 / 0.5). Model-файл их не перекрывает. Почему: лимиты сессии не хранятся в коде.
- **API-ключ из хардкода `API_KEY_XAI` / `API_KEY_GEMINI`** → обязательный `api-key` в model-файле (строка или путь к файлу, резолв в `conf`). Почему: секреты не в коде агентов. Эталон `grok-4.6.json5` не менялся.
- **Не было model-файлов Gemini** → `harness/gemini-3.8-flash.json5`, `gemini-3.5-flash-lite.json5`, `gemini-3.1-flash-lite.json5` с stub `api-key` и `rpm`/`tpm`/`rpd`. Почему: таблица известных моделей; Google без `rpm` — ошибка старта.
- **Путь `git` хардкодился как `"git"`** → `tools.git.executable` обязателен при включённом `git`; `Command::new(executable)`. В рабочем `harness.json5` записано `"git"` как значение конфига, не fallback в коде.
- **CLI через invoke** → `harness.exe --help` / `--prompt` / `--dump` / `--conf-new`; справка из `crates/harness/help.txt` через `include_str`. `--prompt`/`--dump`/`--conf-new` xor; `--conf` по умолчанию `<process-cwd>/harness.json5`.
- **Монолит `DefaultTools`** → модули `fs`/`git`/`search`/`terminal`/`meta`/`user` + общий `sandbox` + хелпер процессов. Registry собирает только ключи из conf.
- **Логгер внутри Python-пакета** → крейт `logger`: trait + `DoubleLogger` + `NullLogger`, в loop/tools как `Arc<dyn Logger>`.
- **Официальных Rust SDK нет** → HTTP `reqwest` (blocking): xAI `chat/completions`, Google `generateContent`, dump/reload свой JSON (совместимость с Python-dump не требуется).

## Сознательные решения по расхождениям в ТЗ

- Отдельного модуля `tools.invoke` / `invoke.rs` нет: `run_invoke` живёт в `terminal` (дерево файлов и список модулей ТЗ). Ключ `invoke` в `tools` — ошибка «unknown tool module».
- У `terminal` обязательны `command-execution-limit` и `invoke-execution-limit` (как в Python-настройках и в пункте «обязательны, если модуль включён»), а не одно поле `execution-limit` из наброска схемы.
- `tools.terminal.cwd-list` заполнен workspace + бывшими w-dirs (в Python cwd команд проверялся через `dirs`).

## Проверка

- `cargo test --workspace` в `./harness`: 12 (conf) + 12 (harness/cli+agents) + 26 (tools) — все зелёные.
- `cargo run -p harness -- --help` — справка с `--conf-new`, код 0.
- Без `--prompt`/`--dump`/`--conf-new` — «нужен --prompt, --dump или --conf-new», код 1.
- `--prompt` вместе с `--dump` — xor-ошибка, код 1.
- `--conf-new` пишет шаблон (комментарии + все модули + заглушки); повтор — «already exists», файл не затирается.
- Живой LLM-вызов не делался (ключи-заглушки).
