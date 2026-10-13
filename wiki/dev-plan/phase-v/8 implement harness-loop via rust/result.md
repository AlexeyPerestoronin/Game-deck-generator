# result

- крейты: `logger` + `tools` → `conf` + `logger` + `tools` (чтение conf вынесено в отдельный крейт по investigation)
- `harness/crates/harness/src/config.rs` → `harness/crates/conf/` (`lib.rs` + `src/str/keys.rs`) (единственная точка чтения json5; ключи и дефолты в одном модуле)
- CLI: `--prompt` | `--dump` → `--prompt` | `--dump` | `--conf-new` (генерация полного json5-шаблона с комментариями, дефолтами и заглушками)
- API-ключи: файлы `API_KEY_XAI` / `API_KEY_GEMINI` в коде агентов → обязательный `api-key` в каждом model-файле (путь к файлу или сам ключ; эталон уже в `harness/grok-4.6.json5`)
- зависимости: бинарник сам парсил conf → бинарник/`tools` берут типизированные настройки из `conf`, json5 сами не открывают
- проверки: добавлены `--conf-new`, отсутствие `api-key`, xor трёх CLI-режимов
