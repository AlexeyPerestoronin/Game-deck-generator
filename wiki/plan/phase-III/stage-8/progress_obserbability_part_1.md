# Задача: «progress observability - part-1»
Сейчас в крейте `deck_gen_wasm` есть внутренний крейт `progress`, который предоставляет удобные макросы для обёртки блоков и циклов с целью наблюдения за прогрессом исполнения кода.

Задача:
1. сделать данный крейт обособленной частью репозитория (создать новый крейт progress_viewer), который будет предоставлять в качестве api интерфейс `ProgressHandler` и макросы для обёртки;
2. крейт `deck_gen` при CLI запуске должен использовать собственную реализацию `ProgressHandler` для всех cli-команд с целью сделать наблюдаемым прогресс выполнения таковых;
3. крейт `deck_gen_wasm` должен использовать собственную реализацию `ProgressHandler` и использовать её для запуска api-команд из `deck_gen_wasm` (передавать в качестве последнего параметра) а не только внутри собственного кода, как это сделано сейчас;
4. публичные api крейтов `prepare_pdf_host` и `prepare_pdf_web` тоже должны получить расширение сигнатуры api методов параметром принимающим реализацию интерфейса `ProgressHandler` и использовать его для отслеживания прогресса выполнения своего кода.

# Что необходимо сделать
Извлечь внутренний крейт `deck_gen_wasm/progress` (package name `deck_gen_wasm_progress`) в самостоятельный крейт репозитория `progress_viewer` (директория `progress_viewer/` в корне, package name `progress_viewer`), который экспортирует интерфейс `ProgressHandler` и макросы обёртки `progress_wrapper!`, `progress_block!`, `progress_loop!`.

Крейт `deck_gen` (при сборке с feature cli) должен создавать собственную реализацию `ProgressHandler` и передавать её в следующие CLI-команды (`html`, `pdf` и `png`) как последний параметр в вызовы `prepare_html_named` / `prepare_pdf_named` / `prepare_png_named` (и внутренние), чтобы прогресс выполнения команд стал наблюдаемым.

Крейт `deck_gen_wasm` (и его sub-компоненты workspace/actions, export, import, template) должен использовать собственную реализацию `ProgressHandler` (текущий с paint-хук) и передавать её как последний параметр при вызовах api-команд из `deck_gen` (в actions.rs), а не только оборачивать свои внутренние операции.

Публичные API крейтов `prepare_pdf_host` (методы Chrome: html_to_pdf_bytes, html_file_to_pdf_*, html_to_png_bytes и т.п.) и `prepare_pdf_web` (WebPdfEngine, WebPngEngine, html_to_* функции) должны быть расширены последним параметром, принимающим реализацию `ProgressHandler`; внутри методов использовать его для установки промежуточных значений прогресса при выполнении генерации PDF/PNG.

Адаптировать существующие вызовы prepare_* внутри deck_gen/lib.rs, чтобы пробрасывать и применять handler (coarse-grained по этапам: load, per-deck render, engine calls). Сохранить обратную совместимость по поведению (noop-handler по умолчанию не обязателен, но изменения минимальны).

# Scope кода (минимальная рабочая область)
- Создать новый крейт: `progress_viewer/Cargo.toml` (с описанием), `progress_viewer/src/lib.rs` (pub use + mod), `progress_viewer/src/progress.rs`, `progress_viewer/src/macros.rs`, `progress_viewer/src/poll.rs` — перенести и минимально адаптировать код из `deck_gen_wasm/progress/` (ввести `pub trait ProgressHandler { fn set(&self, pct: f32); }`; `Progress` реализует его; макросы работают через переданный handler; paint-хук остаётся в wasm-варианте Progress::with_paint).

- `Cargo.toml` (корень workspace): добавить `"progress_viewer"` в массив members (и опционально default-members).

- `deck_gen_wasm/workspace/Cargo.toml`, `import/Cargo.toml`, `export/Cargo.toml`, `template/Cargo.toml`: заменить зависимость `deck_gen_wasm_progress = { path = "../progress" }` на `progress_viewer = { path = "../../progress_viewer" }`; обновить все `use deck_gen_wasm_progress::...` на `use progress_viewer::...` (в т.ч. в тестах).

- `deck_gen/src/lib.rs`: подключить progress_viewer; расширить сигнатуры `prepare_html_named`, `prepare_pdf_named`, `prepare_png_named` (и не-named обёртки) последним параметром (напр. `progress: &impl ProgressHandler` или `progress: impl ProgressHandler + Clone`); внутри циклов по decks и ключевых шагов (catalog/find, render::prepare_html, engine calls) использовать `progress_block!` / `progress_wrapper!` или прямые `.set`; обновить вызовы prepare внутри png/pdf/html путей.

- `deck_gen/src/cli.rs`: в `html_command`/`pdf_command`/`png_command` (и list при необходимости) создавать локальную реализацию `ProgressHandler` (простая структура CliProgress с println или silent для part-1), передавать в prepare_*_named; обновить pollster::block_on вызовы при необходимости.

- `prepare_pdf_host/src/lib.rs` + `chrome/mod.rs`: расширить публичные методы генерации (html_to_pdf_bytes, html_to_png_bytes, html_file_to_*) последним параметром `progress: &dyn ProgressHandler` (или generic); внутри — ставить set(0) перед работой, set(50/100) после ключевых шагов (navigate, print_to_pdf, capture_screenshot); обновить DeleteOnDrop и temp helpers без изменений.

- `prepare_pdf_web/src/lib.rs`: расширить WebPdfEngine / WebPngEngine impl'ы трейтов + внутренние async fn (html_to_pdf, html_to_png, ...) последним параметром progress; пробрасывать/использовать вокруг JS promise вызовов (coarse: перед/после raster); обновить html_to_jpeg_pages.js только если нужно для subprogress (минимально — не трогать).

- `deck_gen_wasm/workspace/src/actions.rs`: в `prepare_html`, `prepare_pdf` (и если появится png) передавать `progress` (из `workspace.progress_handle()`) как последний аргумент в deck_gen::prepare_* вместо/в дополнение к внешней progress_wrapper вокруг всего; сохранить существующие обёртки для других действий (load, zip и т.д.).

- `deck_gen/src/pdf_engine/mod.rs`: при необходимости пробросить progress в prepare_pdf_named (и в вызовы engine.html_to_pdf); CardPngGenerator / PdfEngineGenerator — не расширять (progress идёт через prepare, engine-вызовы оборачиваются внутри prepare).

- Не добавлять новые файлы вне progress_viewer/; не менять архитектуру (FileSystem, engine traits, render pipeline, catalog, load, model, subst, conf остаются без изменений по логике); только добавить параметр и обёртки прогресса + миграция крейта.

- После миграции можно удалить `deck_gen_wasm/progress/` (или оставить на переходный период, но по задаче — обособить).

- Убедиться в сборке: `cargo test -p deck_gen --features cli`, `cargo check -p progress_viewer`, wasm-check соответствующих крейтов, unit-тесты прогресса перенесены и работают.

# Дополнительные указания
1. Изменения в коде должны быть минимальными.
2. Запрещено менять существующую архитектуру.
3. Крейты, код которых подвергался изменениям, должны проходить сборку и проверку локальными unit-тестами.
4. О результатах отпишись в `wiki\result.md` в формате `было→стало(почему)` (очисти файл, если там есть какое-то содержимое перед новой записью).
5. если от меня требуются какие-то действия, то пиши их в `wiki\note.md` (очисти файл, если там есть какое-то содержимое перед новой записью).

## Как правильно писать код:
1. этап-1: решить поставленную задачу и убедиться в её работоспособности;
   - первичный код необходимо писать простой, понятный и прямолинейный, без сложных абстракций;
   - если логика кода сложная, но позволяет написать простые unit-тесты для проверки, их надо написать;
2. этап-2: когда поставленная задача будет решена, код, созданный и зафиксированный на этапе-1, необходимо отрефакторить согласно правилам записанным в `wiki\prompts\refactoring-rules.md`
   - при организации кода придерживайся стиля той кодовой базы в которую вносишь изменения;
3. первый и второй этапы должны делаться раздельно, но в рамках одного промпта (решение → проверка → рефакторинг).

## Как правильно проверять работоспособность:
Соседней папке рядом с папкой проекта находится папка Deck Games (`C:/MyLife/Deck Games`), где находятся игры на которых можно проверять работоспособность.
Работоспособность проверяется на игре new-game (`C:/MyLife/Deck Games/Games/Templates/new-game`).
Должны выполняться следующие уже работающие команды:
1. `deck_gen.exe list`
2. `deck_gen.exe html --game new-game`
3. `deck_gen.exe pdf --game new-game`
4. `deck_gen.exe pdf --game "new-game"`
5. `deck_gen.exe pdf --game new-game --deck deck-1st`
6. `deck_gen.exe pdf --game "new-game" --deck deck-1st`
7. `deck_gen.exe pdf --game new-game --deck "колода №2"`
8. `deck_gen.exe pdf --game new-game --deck "дополнительная колода"`
9. `deck_gen.exe pdf --game "new-game" --deck "дополнительная колода"`
10. `deck_gen.exe png --game new-game`
11. `deck_gen.exe png --game "new-game"`
12. `deck_gen.exe png --game new-game --deck deck-1st`
13. `deck_gen.exe png --game "new-game" --deck deck-1st`
14. `deck_gen.exe png --game new-game --deck "колода №2"`
15. `deck_gen.exe png --game new-game --deck "дополнительная колода"`
16. `deck_gen.exe png --game "new-game" --deck "дополнительная колода"`

## Бережливый подход
1. Максимально береги баланс токенов:
   - использовать X-Search ЗАПРЕЩЕНО;
   - использовать Web-Search ЗАПРЕЩЕНО:
     - если необходимо, сформулируй в чате запрос на разрешение поиска с описанием какую информацию хочешь найти и для чего она нужна в рамках решаемой задачи;