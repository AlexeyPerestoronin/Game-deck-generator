# Результат планирования

было: один `planning.md` со списком аспектов без todo → стало: 6 независимых задач `todo-*.md` (почему: шаблон development.md, scope не пересекаются, связанные UI-правки ключа склеены).

| файл | аспект(ы) | склейка |
|---|---|---|
| `todo-ai-modal-api-key.md` | api key entering + api key generation help | одна модалка, один `ModelConf`, одни json5 |
| `todo-clear-games.md` | clear workspace | изолированный `Workspace::clear` |
| `todo-requests-per-minute.md` | RPS→RPM и `-1` | парсер + throttle, без retry |
| `todo-http-503-retry.md` | fail 409/503 | только retry/notify, `throttle` не трогать |
| `todo-help-locale.md` | ai-help.md + game/user-help + `help/` | все help одним install-контрактом |
| `todo-ai-prompt-templates.md` | templates to file | только bundled промпты, не VFS help |

Порядок выполнения любой: в каждой задаче явно запрещены чужие поля/файлы (json5: одна задача пишет `api-key-hosting`, другая только переименовывает RPM; `engine.rs` режется по функциям).
