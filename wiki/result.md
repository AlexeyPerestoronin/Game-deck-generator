# Отчёт: картинки в `deck_gen_wasm`

Формат: было → стало (почему).

## Конфиг импорта
- Было: `ALLOWED_EXTENSIONS = md, json, json5, html, scss`, лимита размера нет.
- Стало: рядом `IMAGE_EXTENSIONS = jpg, jpeg, png, ico, icon` и `MAX_IMAGE_BYTES = 100 MiB`.
- Почему: задача требует jpg/png/icon и потолок 100 МБ; jpeg/ico — те же форматы под обычными расширениями.

## Политика файлов
- Было: `extension_allowed` только по текстовому списку; reject-текст захардкожен.
- Стало: текст + картинки; `classify(path, size)` → Text / Image / Rejected / Oversized; сообщения собираются из констант.
- Почему: один источник правды для Load Game и «load file(s)», unit-тесты без DOM.

## Загрузка папки с игрой
- Было: все файлы читались как UTF-8 `String`; чужое расширение валило весь импорт.
- Стало: картинки читаются как байты (`FileBody::Bytes`) и кладутся в VFS через `put_bytes`; превышение 100 МБ — модалка со списком файлов; прочие расширения по-прежнему валят импорт.
- Почему: PNG/JPEG/ICO не UTF-8; бинарный узел VFS уже был для PDF.

## Пункт ПКМ `load file(s)`
- Было: у папки только Rename / Delete.
- Стало: первый пункт `load file(s)` открывает `<input type="file" multiple accept=…>` с `ALLOWED_EXTENSIONS` + картинками; файлы пишутся в кликнутую папку; ошибка расширения/размера — модалка в explorer.
- Почему: так сформулирована задача; accept режет диалог ОС, серверная проверка та же, что у папки.

## Preview
- Было: Preview в меню и панели только для html/md/pdf.
- Стало: jpg/jpeg/png/ico/icon тоже Preview; панель рисует `<img>` с blob URL (тот же helper, что у PDF).
- Почему: как у уже существующих preview-типов; клик по файлу по-прежнему открывает edit (для binary — заглушка с размером).

## Скрытый file input
- Было: разметка `<input>` продублирована в directory-picker.
- Стало: общий `load_folder/input.rs`; папка и «load file(s)» только конфигурируют input.
- Почему: одна ответственность, без новой абстракции «на будущее».

## Инсталляция в VFS
- Было: `install_folder(..., files: &[(String, String)])`.
- Стало: `FileBody` + `install_files` для целевой папки (имя файла без подпутей).
- Почему: текст и картинки в одном проходе; path traversal с `file.name()` отсекается `file_name()`.

## UI-тексты
- Было: confirm Load Game: «only md, json, json5, html and scss».
- Стало: явно перечислены картинки и 100 MB.
- Почему: пользователь должен видеть актуальные правила до выбора папки.

## Тесты
- Было: 25 тестов, картинок нет.
- Стало: 28; политика расширений/размера/`classify`; install text+png; меню Preview и `load file(s)`.
- Почему: логика классификации и записи в VFS проверяется без браузера.

## Сборка
- `cargo test -p deck_gen_wasm`: 28 passed.
- `cargo check -p deck_gen_wasm --target wasm32-unknown-unknown`: ok.

## Сознательно не трогали
- Другие крейты (`deck_gen`, PDF engine) — по todo анализировать только wasm-крейт.
- Persist: binary (как PDF) не пишутся в localStorage — 100 МБ туда нельзя.
- Авто-preview по клику на картинку — у md/html/pdf клик тоже открывает edit.
