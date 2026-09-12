# Доработка help и шаблона new-game

Краткий отчёт в формате было→стало(почему). Менялся только крейт `deck_gen_wasm`.

## Когда качается `user-help.md`

- **Было:** fetch только если `load_session()` = `None` (самый первый заход в браузер).
- **Стало:** `help::needs_download` — качаем, если в VFS нет `deck_gen_wasm/user-help.md`.
- **Почему:** после Clear сессия уже есть, но файла нет; help должен появиться снова при следующей загрузке страницы.

- **Было:** `App` вызывал `open_first_visit_help` только на first visit.
- **Стало:** каждый старт вызывает `ensure_user_help`.
- **Почему:** условие — наличие файла в дереве, не факт «ключа localStorage не было».

## Когда открывается preview

- **Было:** после скачивания help всегда открывалась вкладка Preview.
- **Стало:** Preview только если `tabs` пустой (`help::should_open_preview`).
- **Почему:** если уже открыт редактор (selected-файл из сессии), не перебивать его help’ом.

- **Было:** help в VFS и пустые вкладки не открывали preview (fetch не шёл).
- **Стало:** файл на месте + нет вкладок → сразу `open_help_preview`, без сети.
- **Почему:** пункт 2 не зависит от пункта 1.

## Шаблон игры без копии в Trunk

- **Было:** `index.html` копировал `../games/new-game` и `../games/conf.json5` в `template/`.
- **Стало:** этих `copy-dir` / `copy-file` нет; остался только `user-help.md`.
- **Почему:** игровые данные не должны ехать с бандлом, только с GitHub.

- **Было:** `load_template_files` при дырявом GitHub брал локальный `/template/…`.
- **Стало:** только `list_template_blob_paths` + `fetch_listed_blobs`; нет `load_from_local`.
- **Почему:** локальной копии больше нет; fallback скрывал бы ошибку сети.

- **Было:** `LOCAL_SOURCE_LABEL` в `conf::template`.
- **Стало:** константа удалена; `InstalledGame.source` всегда `GITHUB_SOURCE_LABEL`.
- **Почему:** висячая подпись источника.

- **Было:** ошибка New game писалась только в status footer.
- **Стало:** тот же `AlertModal`, что у prepare/load; заголовок «Cannot load new-game».
- **Почему:** в задаче — модальное предупреждение, если GitHub не отдал шаблон.

- **Было:** `NewGameButton` без `warning`.
- **Стало:** как `PrepareHtmlButton`: прокидывает `warning` / `warning_title`.
- **Почему:** модалка живёт в activity bar, кнопка только выставляет заголовок и зовёт action.

## Help: локальный fallback

- **Было:** GitHub, иначе Trunk `user-help.md`.
- **Стало:** то же (copy-file help в `index.html` не трогали).
- **Почему:** задача запретила копировать `../games/`, не help.

## Тесты и комментарии

- **Было:** не было явных правил «качать / открывать».
- **Стало:** `needs_download` / `should_open_preview` + тест `download_when_missing_preview_when_no_tabs`.
- **Почему:** две независимые булевы политики, их можно проверить без HTTP.

- **Было:** комментарии про first visit и local template.
- **Стало:** `help.rs`, `template.rs`, `http.rs`, `github.rs`, `app.rs` описывают GitHub-only шаблон и help-if-missing.
- **Почему:** шапки модулей по правилам рефакторинга.

## Что не меняли

- **Было:** сессия не хранит список вкладок; `initial_tabs` открывает Edit, если selected — файл.
- **Стало:** без изменений.
- **Почему:** «нет вкладок» после reload бывает, когда selected не файл.

- **Было:** прочие крейты.
- **Стало:** не трогались.
- **Почему:** так сказано в `wiki/todo.md`.

## Проверки

- **Было:** 22 unit-теста.
- **Стало:** 23; `cargo test -p deck_gen_wasm` ok; wasm32 check ok.
- **Почему:** изменённый крейт должен собираться и проходить локальные тесты.
