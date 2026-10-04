# result-help-locale

- `user-help.md` / `game-help.md` / `ai-models/ai-help.md` в корне VFS → `help/<name>-<en|ru>.md` (все help в одной папке; старые пути больше не инсталлируются, миграции нет)
- bundled `ai-help.md`, `user-help.md`, `game-help.md` → пары `*-en.md` + `*-ru.md` (EN = прежний текст с поправленными путями; RU = полный перевод)
- `conf::help::PATH` / `game_help::PATH` / `ai::HELP` (константы) → `help::path(locale)`, `game_help::path(locale)`, `ai::help(locale)` (пути не размазаны по UI)
- `ensure_user_help` / `ensure_ai_files` / `install_*` без локали → смотрят текущий `en`/`ru` и копируют **только** недостающий файл (или HTML-shell); другая локаль в VFS не удаляется; неизвестный язык не выдумывается
- агент читал `game-help.md` → читает `help/game-help-<locale>.md` той же локали, что и промпт
- превью при пустых вкладках открывало `user-help.md` → открывает `help/user-help-<locale>.md`
