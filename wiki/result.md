# Help только локально, в корне VFS

Краткий отчёт в формате было→стало(почему). Менялся только `deck_gen_wasm`. Задача на доработку №2.

## Источник текста

- **Было:** `fetch_user_help` сначала GET GitHub raw, иначе `include_str`.
- **Стало:** только `bundled_help()` = `include_str!("../user-help.md")`. Сети нет.
- **Почему:** пункт 1 — help не должен приходить с GitHub.

- **Было:** `ensure_user_help` ставил `loading`, `spawn_local`, ждал HTTP.
- **Стало:** синхронный `put_file` бандла в VFS.
- **Почему:** копирование строки из WASM не async; нечего блокировать кнопки.

- **Было:** `github.rs` собирал raw URL для help; тест `help_raw_url_uses_master_and_repo_path`.
- **Стало:** help из GitHub-модуля убран; тест удалён.
- **Почему:** висячая связь help ↔ GitHub.

- **Было:** `http.rs` описан как «template + help».
- **Стало:** только template blobs.
- **Почему:** help больше не вызывает `fetch_text`.

## Путь в дереве

- **Было:** `conf::help::PATH` = `deck_gen_wasm/user-help.md` (папка под крейт).
- **Стало:** `user-help.md` в корне workspace.
- **Почему:** пункт 2 — «в самом корне» локальной (виртуальной) ФС.

- **Было:** `install_user_help` создавал каталог `deck_gen_wasm/`.
- **Стало:** файл лежит рядом с `games/`, без лишней папки.
- **Почему:** корень VFS — пустой prefix у `put_file("user-help.md")`.

- **Было:** тест ждал `is_dir("deck_gen_wasm")`.
- **Стало:** `installs_at_workspace_root` проверяет `user-help.md` и отсутствие `deck_gen_wasm`.
- **Почему:** регресс пути.

## Имена

- **Было:** `needs_download`.
- **Стало:** `needs_install`.
- **Почему:** это не HTTP download, а копия бандла.

- **Было:** в help.md «from GitHub master».
- **Стало:** «`user-help.md` at the workspace root (shipped with the app)».
- **Почему:** текст не должен врать про источник.

## Что оставили

- **Было:** HTML-оболочка в VFS считалась битым help и переустанавливалась.
- **Стало:** то же для корневого `user-help.md`.
- **Почему:** старые сессии могли хранить index.html; не тащить его как help.

- **Было:** preview, если вкладок нет.
- **Стало:** без изменений.
- **Почему:** не входит в №2.

- **Было:** New game с GitHub + модалка.
- **Стало:** без изменений.
- **Почему:** игровые данные по-прежнему с master.

## Проверки

- **Было:** 25 тестов (включая GitHub URL help).
- **Стало:** 24; `cargo test -p deck_gen_wasm` ok; wasm32 check ok.
- **Почему:** изменённый крейт должен собираться и проходить unit-тесты.
