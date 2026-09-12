# Отчёт: persist PDF и картинок после reload

Формат: было → стало (почему). Доработка №2 из `wiki/todo.md`.

## Симптом
- Было: после F5 текст в дереве на месте, PDF и картинки исчезают.
- Стало: бинарники поднимаются из IndexedDB до первого кадра explorer.
- Почему: `Session::from_workspace` вызывал `without_binaries()` из‑за квоты localStorage.

## Где лежат байты
- Было: только localStorage JSON; `Node::Binary` вырезался при снимке.
- Стало: текст/selection/expanded — по‑прежнему localStorage; PDF/картинки — IndexedDB `deck_gen_wasm` / store `binaries` / ключ `files`.
- Почему: 100 МБ в localStorage не влезают; IDB держит `Uint8Array` без JSON-раздувания.

## Загрузка приложения
- Было: `App` сразу `from_session(load_session())` и монтировал панели.
- Стало: короткий «Loading workspace…», `load_binaries`, `restore_binaries`, затем `LoadedApp`.
- Почему: иначе autosave мог записать пустой IDB до чтения и стереть файлы.

## Autosave
- Было: Effect на каждый сигнал писал весь JSON (без binary).
- Стало: текст — как раньше; IDB put только если `binaries_fingerprint` сдвинулся.
- Почему: не гонять 100 МБ в IDB на каждый символ в md.

## Ручной Save / Clear
- Было: Save и Clear трогали только localStorage.
- Стало: Save ждёт `save_binaries`; Clear пишет пустой список в IDB.
- Почему: иначе Clear оставлял бы PDF в базе до следующего fingerprint.

## VFS
- Было: `without_binaries` / `put_bytes` / `is_binary`, без списка и restore.
- Стало: `binary_entries()` и `restore_binaries()`; strip для JSON не тронут.
- Почему: persist не должен знать обход дерева.

## Конфиг
- Было: только `STORAGE_KEY`.
- Стало: `IDB_NAME`, `IDB_STORE`, `IDB_KEY`, `IDB_VERSION` в `conf::session`.
- Почему: правило «не мутабельные статики — константы в conf».

## Модули
- Было: один `persist.rs`.
- Стало: `persist/mod.rs` (JSON) + `persist/binaries.rs` (IDB).
- Почему: один модуль — одна ответственность.

## Тесты
- Было: 29 тестов, restore binary не проверялся.
- Стало: 31; `binaries_restore_onto_stripped_tree`; fingerprint меняется от path/bytes.
- Почему: IDB в unit-тестах нет; дерево и hash проверяются без браузера.

## Сборка
- `cargo test -p deck_gen_wasm`: 31 passed.
- `cargo check -p deck_gen_wasm --target wasm32-unknown-unknown`: ok.

## Cargo.toml
- Было: web-sys без IDB.
- Стало: IdbFactory / Database / ObjectStore / Request / Transaction + Event, DomStringList.
- Почему: без новых крейтов, тот же web-sys.

## Не меняли
- Алгоритм strip для JSON — квота localStorage та же.
- Форматы картинок и preview — вне этой доработки.
