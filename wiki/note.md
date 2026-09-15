# Пост-действия (требуются от пользователя)

После применения изменений по todo.md (текущая задача: *.j2 загрузка + синтаксис js/j2):

1. Ручная проверка в браузере (обязательно, т.к. unit-тесты не заменяют реальный File Picker и не показывают реальную подсветку):
   - `serve.bat` (или `trunk serve`)
   - Открыть http://127.0.0.1:8080
   - Загрузить папку, содержащую .j2 (например, скопировать games/monopoly-2.0/views/face-layout.j2 в новую игру или использовать существующие views после load).
   - Загрузить .js напрямую или из папки.
   - Открыть .js файл в редакторе — должна быть цветная подсветка (не plain textarea). Открыть .j2 — либо подсветка (если syntect знает "j2"), либо plain (graceful).
   - Проверить, что .j2 и .js видны в explorer (как обычные файлы, без специальной иконки — это ок).
   - Проверить отклонение других расширений.
   - Desktop + (при наличии) mobile viewport.
   - Проверить consistency: после load, edit, save, экспорт — файлы остаются.

2. Полноценная иконка/классификация для .js и .j2 (ExplorerIcon, FileKind, preview и т.д.):
   - Можно сделать позже (добавив варианты в enum'ы fs), но не требовалось текущей задачей. Сейчас они Other + syntax для highlight/editor.

3. Полная проверка (опционально):
   - cargo test -p deck_gen_wasm_fs -p deck_gen_wasm_import -p deck_gen_wasm_ui
   - trunk build (или serve)
   - (вне инструкции) можно проверить и deck_gen CLI, но анализ/правки были только по deck_gen_wasm.

4. Зафиксировать изменения git'ом (пример):
   - git add deck_gen_wasm/conf/src/api.rs deck_gen_wasm/fs/src/file_kind.rs deck_gen_wasm/import/src/policy.rs deck_gen_wasm/ui/src/windows/editor/highlight.rs deck_gen_wasm/ui/Cargo.toml wiki/result.md wiki/note.md

5. Если в будущем добавлять другие текстовые типы ( .ts, .mdx и т.п.) — одна строка в IMPORT_TEXT + тест в policy/fs + (при необходимости) строка в syntax_name.

Никаких других действий не требуется от пользователя для завершения этой задачи. Изменения изолированы в deck_gen_wasm*, код простой, тесты+build+trunk-build подтверждены.
