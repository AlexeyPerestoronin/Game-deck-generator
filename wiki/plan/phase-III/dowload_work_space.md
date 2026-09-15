# Задача
Кнопка выгрузки рабочего пространства (Download в Activity Bar) по нажатию открывает два действия, а не сразу качает ZIP:
1. выгрузить как ZIP-архив (уже реализовано);
2. выгрузить в существующую папку на диске с заменой её содержимого деревом текущего VFS.

## Текущее поведение (факт из кода)
- UI: `DownloadButton` (`deck_gen_wasm/ui/src/buttons/download.rs`) — один клик → `workspace.download()`.
- `Workspace::download` (`deck_gen_wasm/workspace/src/actions.rs`): `flush_draft` → `vfs_to_zip` → `save_zip_bytes`.
- ZIP: `deck_gen_wasm/export/src/zip.rs` (`vfs.visit_entries`, DEFLATE, имя `conf::export::ZIP_FILENAME` = `workspace.zip`).
- Отдача ZIP: `deck_gen_wasm/export/src/browser.rs` — `showSaveFilePicker`, иначе скрытый `<a download>`.
- Запись дерева в произвольную папку диска отсутствует. `showDirectoryPicker` есть только на чтение в `deck_gen_wasm/import/src/picker.rs` (Load Game) — этот путь не использовать для экспорта.
- JS FFI: `deck_gen_wasm_browser` (`has_window_fn`, `call_async_js`). Обход VFS: `Vfs::visit_entries` (`path`, `None` = каталог, `Some(bytes)` = файл).

## UI: выдвижная панель, не сразу download
- Первый клик по Download **не** стартует выгрузку: справа от кнопки выезжает виджет (слева направо, поверх Explorer).
- Внутри виджета **две кнопки в одну горизонтальную линию** (не вертикальное контекстное меню Explorer).
- Подписи кнопки (короткие, английские, в стиле остального UI), например: `ZIP` и `Folder`.
- Повторный клик по Download или клик снаружи закрывает виджет без действия.
- Выбор пункта выполняет действие и закрывает виджет.
- Тултип кнопки сменить: сейчас «Download the workspace as a ZIP archive.»
- Тултипы для подкнопок придумать и добавить.

Не копировать `ContextMenu` Explorer. Это локальный flyout у `DownloadButton` (как `DelayedTooltip` позиционируется от `.tooltip-host`: `left: calc(100% + …)`). CSS — в `deck_gen_wasm/style.css`.

## Действие ZIP
Оставить текущий пайплайн: `flush_draft` → `vfs_to_zip` → `save_zip_bytes`. Вынести вызов в обработчик пункта «ZIP» вместо прямого `on:click` кнопки.

## Действие Folder (замена содержимого)
1. `flush_draft`.
2. `showDirectoryPicker({ mode: "readwrite" })`. Отмена пикера — тихий выход, статус не ошибка.
3. Перед записью — `ConfirmModal` в том же паттерне, что Clear/Load Game в `ActivityBar`: заголовок/текст про полную замену содержимого выбранной папки; Cancel — не писать.
4. После подтверждения:
   - удалить всё текущее содержимое выбранной папки;
   - записать **весь** VFS (корень workspace, включая `user-help.md` и `games/…`) относительными путями: каталоги через `getDirectoryHandle({ create: true })`, файлы через `getFileHandle` + `createWritable` + `write` + `close`.
5. Статус: успех / текст ошибки. `loading` Activity Bar — как у остальных долгих действий workspace.
6. Нет `showDirectoryPicker` (Firefox и т.п.): **не** падать в ZIP. Статус/alert: folder-export только в Chromium с File System Access API; ZIP по-прежнему доступен первым пунктом.

Пикер и запись — в крейт `deck_gen_wasm_export` (рядом с ZIP), не в `import`. `Workspace` получает метод вроде `export_to_folder` рядом с `download`. Import/picker/read не менять.

## Не делать
- Не подменять autosave/persist (localStorage) выгрузкой на диск.
- Не менять формат ZIP и `vfs_to_zip`, кроме как вызывать их из пункта ZIP.
- Не писать в папку без подтверждения.
- Не тащить Load Game / `pick_and_read_folder` в экспорт.

## Приёмка
- Клик Download → выезд двух пунктов в ряд; без выбора ничего не скачивается.
- ZIP — как сейчас (`workspace.zip` или save picker).
- Folder в Chrome: пикер папки → confirm → содержимое папки = дерево VFS, старые файлы папки удалены.
- Отмена пикера или confirm — диск не меняется.
- Без Directory Picker пункт Folder сообщает, что недоступен; ZIP работает.

***

Activity Bar узкий (48px, кнопки `.activity-btn` 40×40). Flyout должен перекрывать Explorer, не раздувая колонку грида `.ide`.

# Особые указания
1. Изменения в коде должны быть минимальными!
2. Запрещено менять существующую архитектуру!
3. Какой код использовать для анализа:
   - крейт `deck_gen_wasm_ui`:
     - `deck_gen_wasm/ui/src/buttons/download.rs`
     - `deck_gen_wasm/ui/src/bars/activity.rs` (ConfirmModal / warning — как у Clear/Load Game)
     - `deck_gen_wasm/ui/src/modals/confirm.rs` (переиспользовать, не копировать разметку)
     - `deck_gen_wasm/style.css` (`.activity-bar`, `.activity-btn`, `.tooltip-host`, `.tooltip`; новые классы flyout)
   - крейт `deck_gen_wasm_workspace`:
     - `deck_gen_wasm/workspace/src/actions.rs` (`download`, `flush_draft`, `loading`/`finish_async`)
   - крейт `deck_gen_wasm_export`:
     - `deck_gen_wasm/export/src/api.rs`
     - `deck_gen_wasm/export/src/browser.rs`
     - `deck_gen_wasm/export/src/zip.rs` (только как готовый ZIP-путь)
     - `deck_gen_wasm/export/Cargo.toml`
   - крейт `deck_gen_wasm_browser`: `deck_gen_wasm/browser/src/js.rs` (`has_window_fn`, `call_async_js`) — вызывать, не дублировать Reflect;
   - крейт `deck_gen_wasm_fs`: только `Vfs::visit_entries` в `deck_gen_wasm/fs/src/vfs.rs` (сигнатура обхода);
   - крейт `deck_gen_wasm_conf`: только `conf::export` в `deck_gen_wasm/conf/src/api.rs` (при необходимости константа имени ZIP уже есть);
   - прочие файлы и папки репозитория ИГНОРИРУЙ;
3. Краткий отчёт (50-100 строк) о проделанной работе в формате `было→стало(почему)` напиши в `wiki\result.md`.
4. Если какие-то пост-действия требуются от меня напиши их в `wiki\note.md`.

# Дополнительные указания

## Как убедиться в правильности решения:
1. крейты, код которых подвергался изменениям, должны проходить сборку и проверку локальными unit-тестами;
   - `cargo test -p deck_gen_wasm_export --lib`
   - `cargo test -p deck_gen_wasm_workspace --lib`
   - `cargo test -p deck_gen_wasm_ui --lib`
   - `cargo check -p deck_gen_wasm`

## Как правильно писать код:
1. этап-1: решить поставленную задачу и убедиться в её работоспособнои требуемым образом;
   - код необходимо писать простой, понятный и прямолинейный, без сложных абстракций;
   - если логика кода сложная, но позволяет написать простые unit-тесты для проверки, их надо написать;
2. этап-2: когда поставленная задача будет решена, код, созданный и зафиксированный на этапе-1, необходимо отрефакторить согласно правилам записанным в `wiki\prompts\refactoring-rules.md`
   - при организации кода придерживайся стиля той кодовой базы в которую вносишь изменения;

## Бережливый подход
1. Максимально береги баланс токенов:
   - использовать X-Search ЗАПРЕЩЕНО;
   - использовать Web-Search ЗАПРЕЩЕНО:
     - если необходимо, сформулируй в чате запрос на разрешение поиска с описаем какую информацию хочешь найти и для чего она нужна в рамках решаемой задачи;
