# Почему в preview был index.html, и что исправлено

Краткий отчёт в формате было→стало(почему). Менялся только `deck_gen_wasm`.

## Почему так случилось

- **Было:** GitHub `raw` для `deck_gen_wasm/user-help.md`, при ошибке GET относительного `user-help.md` на Trunk.
- **Стало:** то же GitHub, но локальный HTTP больше не используется.
- **Почему:** локальный GET и вернул тот HTML, который вы видели.

Цепочка:

1. GitHub, скорее всего, не отдал файл (его ещё нет на `master`, сеть, CORS) → код пошёл в fallback.
2. `data-trunk rel="copy-file" data-target-path="user-help.md"`: у Trunk `data-target-path` — **каталог**, не имя файла. В `dist` получилось `user-help.md/user-help.md`, не `dist/user-help.md`.
3. Запрос `GET /user-help.md` попал в каталог / неизвестный путь. Dev-сервер Trunk в SPA-режиме отвечает **HTTP 200** и телом `index.html` (websocket overlay `localhost.:8080` — это он).
4. `fetch_text` считает любой 200 успехом → HTML записали в VFS как Markdown.
5. Preview честно показал «исходник». Сессия сохранила яд в localStorage.

## Fetch

- **Было:** `Ok(body)` с GitHub или с `/user-help.md` без проверки содержимого.
- **Стало:** тело с `<!DOCTYPE html` / `<html` отбрасывается; запас — `include_str!("../user-help.md")` в WASM.
- **Почему:** бандл не ходит на Trunk HTTP и не может подменить help оболочкой приложения.

- **Было:** `conf::help::LOCAL_URL`.
- **Стало:** константа удалена.
- **Почему:** локального URL больше нет.

## copy-file

- **Было:** `index.html` копировал `user-help.md` в `data-target-path="user-help.md"`.
- **Стало:** этой строки нет.
- **Почему:** копирование было сломано семантикой Trunk и больше не нужно.

## Уже отравленный VFS

- **Было:** `needs_download` = «файла нет».
- **Стало:** качаем и если файла нет, и если тело — HTML-документ.
- **Почему:** иначе после фикса reload снова открывал бы сохранённый index.html.

## Тесты

- **Было:** не отличали Markdown от HTML.
- **Стало:** `html_shell_is_not_help`, `redownload_if_stored_help_is_html`; бандл не HTML.
- **Почему:** регресс «SPA 200» ловится без браузера.

## Что не меняли

- **Было:** GitHub первым источником help.
- **Стало:** по-прежнему первый; бандл только если raw пустой/HTML/ошибка.
- **Почему:** как у шаблона игры: master — канон.

- **Было:** New game только с GitHub + модалка.
- **Стало:** без изменений.
- **Почему:** баг только в help fallback.

## Проверки

- **Было:** 23 теста.
- **Стало:** 25; `cargo test -p deck_gen_wasm` ok; wasm32 check ok.
- **Почему:** изменённый крейт должен собираться и проходить unit-тесты.
