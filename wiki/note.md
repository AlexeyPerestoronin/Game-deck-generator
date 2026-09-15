# Рефакторинг №1: «Анализ deck_gen_wasm/fs: неиспользуемый публичный API, инкапсуляция Vfs/VfsFs, обобщение файловых операций»

## Описание проблемы

Необходимо выполнить анализ (без изменения кода) крейта `deck_gen_wasm_fs` (папка `deck_gen_wasm/fs/`). Анализировался **только код внутри `deck_gen_wasm/`** (все остальные файлы и папки репозитория игнорировались согласно указаниям).

Специальные цели:
1. Найти API-методы (и другие публичные элементы), которые экспортируются, но не используются внешними потребителями (крейты-потребители внутри deck_gen_wasm: `workspace`, `ui`, `import`, `export`, `persist`, `template`, `browser`, `conf` и верхнеуровневый `main`).
2. Предложить способы избавиться от необходимости экспонировать `vfs::Vfs` и `vfs_fs::VfsFs` наружу, оставив только `vfs::Node` (в идеале — только `Node` + свободные функции `mkdir`, `create_file`, `put_file`, `read_file`, `visit_entries` и т.п.).
3. Предложить способы убрать из API код, специфичный для бинарных файлов (`visit_binaries`, `restore_binaries`, `without_binaries`, `is_binary`, `put_bytes`/`read_bytes` vs `put_file`/`read_file`). API должен предоставлять общие операции над файлами; определение типа файла и способ работы с ним (текстовый/бинарный, localStorage vs IndexedDB) должно быть в коде вызывающих потребителей.

Использованные методы анализа: чтение исходных файлов + targeted grep по поддиректориям deck_gen_wasm (без fs при поиске внешних использований).

## Результаты анализа публичного API

Публичный API крейта (см. `fs/src/api.rs`, `lib.rs`, `path.rs`, `vfs.rs`, `vfs_fs.rs`, `file_kind.rs`):

**Реэкспортируемые элементы пути (path):**
- `file_ext`, `file_name`, `join_path`, `parent_path`, `path_is_or_under`, `retain_not_under`, `rewrite_prefix`, `rewrite_set`, `split_path`, `unique_name`

**Типы VFS:**
- `Node`, `Vfs`, `VfsFs`

**Модуль kind (file_kind):**
- `FileKind`, `ExplorerIcon`
- `kind_of`, `is_image`, `is_previewable`, `can_highlight`, `syntax_name`, `image_mime`, `extension_allowed`, `explorer_icon`

**Публичные методы Vfs (основные):**
- `mkdir`, `create_file`, `write_file`, `put_file`, `put_bytes`, `exists`, `read_file`, `read_bytes`, `is_binary`, `without_binaries`, `visit_binaries`, `visit_entries`, `restore_binaries`, `is_dir`, `is_file`, `children`, `remove`, `rename`, `copy_entries_into`
- `Vfs::default()`

**Публичные методы VfsFs:**
- `new`, `into_vfs`, `clone_vfs` (+ реализация `deck_gen::fs::FileSystem`)

### Пункт 1: Неиспользуемые экспортируемые элементы

Внешние потребители (поиск по `deck_gen_wasm/` без `fs/**`):

**Не используются нигде за пределами fs-крейта (только внутри fs/src и его тестов):**
- `file_ext` — вызывается исключительно внутри `file_kind.rs` (для `kind_of`).
- `split_path` — используется только внутри `vfs.rs` (все разборы путей) и тестах `path.rs`.
- `Node` — тип экспортируется (`pub use crate::vfs::Node`), но **ни один потребитель** не импортирует `deck_gen_wasm_fs::Node`, не делает pattern match на `Node::File`/`Node::Binary`/`Node::Dir` и не конструирует его напрямую. Вся работа с деревом идёт исключительно через методы `Vfs` (или косвенно через workspace-обёртки).

**Используются внешними потребителями (примеры):**
- Path-хелперы: `join_path`, `parent_path`, `file_name`, `unique_name`, `path_is_or_under`, `retain_not_under`, `rewrite_prefix`, `rewrite_set` — активно в `workspace/state.rs`, `workspace/copy_plan.rs`, `import/*`, `ui/*`, `template/*`.
- `Vfs` — везде (`RwSignal<Vfs>`, `&mut Vfs`, `&Vfs`): `workspace/state.rs`, `workspace/actions.rs`, `workspace/commands.rs`, `persist/session.rs` + `binaries.rs`, `import/install.rs`, `template/*`, `export/zip.rs`, `ui/app.rs`, `ui/windows/*`.
- `VfsFs` — только `workspace/actions.rs` (конструкция + извлечение после `prepare_html`/`prepare_pdf`).
- Все `kind::*` — используются:
  - `kind_of` + `FileKind` — `ui/windows/editor/preview.rs`
  - `is_previewable` — `workspace/split.rs` (и реэкспорт)
  - `extension_allowed`, `is_image` — `import/policy.rs`
  - `can_highlight`, `syntax_name` — `ui/windows/editor/highlight.rs`
  - `image_mime` — `ui/windows/editor/preview.rs`
  - `explorer_icon` + `ExplorerIcon` — `ui/icons/files.rs`
- Методы Vfs (внешние вызовы в src/, не только тесты):
  - `mkdir`, `create_file`, `write_file`, `put_file`, `put_bytes`, `exists`, `read_file`, `read_bytes`, `is_dir`, `is_file`, `is_binary`, `children`, `remove`, `rename`, `copy_entries_into`
  - `without_binaries` — `persist/session.rs`
  - `visit_binaries` — `persist/binaries.rs`
  - `visit_entries` — `export/zip.rs`
  - `restore_binaries` — `ui/app.rs`
- `VfsFs::new` / `into_vfs` / `clone_vfs` — `workspace/actions.rs`

Вывод по п.1: `file_ext`, `split_path` и `Node` можно безопасно убрать из публичного API (сделать `pub(crate)` или полностью внутренними). Остальное — реально используется.

### Пункт 2: Избавление от экспонирования Vfs и VfsFs

Сейчас:
- `Vfs` — центральный тип состояния, живёт в сигналах Leptos, передаётся между слоями (import/install, template, persist, export, ui, workspace).
- `VfsFs` — узкий адаптер только для вызова `deck_gen::prepare_html` / `prepare_pdf` (через `Arc<dyn FileSystem>`). Используется только в `workspace/actions.rs`.

**Предлагаемые решения:**

**Решение №1 (рекомендуемое по духу задачи — только Node + функции):**  
Сделать корневой элемент `Node::Dir` (унифицировать root). Полностью удалить структуру `Vfs`.  
Экспортировать:
- `Node`
- path-хелперы (только используемые)
- `kind::*`
- свободные функции: `mkdir(node: &mut Node, path)`, `create_file`, `put_file`/`put_bytes` (или единый `write_bytes`), `read_file`, `read_bytes`, `exists`, `is_dir`/`is_file`, `children`, `remove`, `rename`, `copy_entries_into`, `visit_entries` и т.д.

Потребители меняют `RwSignal<Vfs>` → `RwSignal<Node>`, `&mut Vfs` → `&mut Node`.  
Для `VfsFs`: не экспортировать тип вообще. Предоставить инкапсулирующую функцию в fs-крейте:

```rust
pub fn with_fs<R>(root: Node, f: impl FnOnce(Arc<dyn deck_gen::fs::FileSystem>) -> R) -> (R, Node)
```

В `workspace/actions.rs` использовать `with_fs(...)` без упоминания `VfsFs` в публичном/пользовательском коде.

**Решение №2:**  
Оставить `Vfs` как newtype-обёртку над `BTreeMap` / `Node`, но сделать поля приватными и не реэкспортировать `Vfs` как публичный тип везде. Передавать через workspace API. `VfsFs` переместить в `workspace` (как деталь реализации prepare). fs-крейт экспортирует только `Node` + операции.

**Решение №3:**  
Сделать `Vfs` и `VfsFs` полностью непрозрачными (opaque types). Добавить методы только на них, но типы всё равно придётся называть в сигнатурах install_* и т.п. Минимальное изменение.

### Пункт 3: Избавление от специального кода для бинарных файлов

Сейчас бинарность "зашита" в ядро:
- `Node::Binary { data: Vec<u8> }` vs `Node::File { content: String }`
- Специальные `put_bytes`/`read_bytes`/`is_binary`
- `visit_binaries` + `restore_binaries` + `without_binaries` (для localStorage + IndexedDB split в persist)
- В `VfsFs` тоже поддержка bytes

Используется только persist (fingerprint/encode/save/load binaries) + ui/app (восстановление) + import (images как bytes) + preview (read_bytes для pdf/image).

**Предлагаемые решения:**

**Решение №1 (основное, соответствует цели):**  
Упростить `Node` до общего вида (все файлы — байты):

```rust
pub enum Node {
    File { data: Vec<u8> },
    Dir { children: BTreeMap<String, Node> },
}
```

Убрать из публичного API:
- `visit_binaries`, `restore_binaries`, `without_binaries`, `is_binary`
- Разделение `put_file`/`put_bytes`, `read_file`/`read_bytes` → единые `write` / `read_bytes` (+ опционально `read_text` который делает `String::from_utf8`).

Оставить/предоставить только общий:
- `visit_entries(&self, visit: impl FnMut(&str, Option<&[u8]>))`

Определение "бинарности" (что отправлять в IndexedDB, что хранить в localStorage, что обрезать при snapshot) полностью выносится в потребителей:
- `persist` использует `kind::is_image` / `kind::...` + `visit_entries`, чтобы решить, какие байты кодировать в IDB, а какие (текстовые) оставить в `Session.vfs`.
- При создании snapshot persist сам строит "лёгкую" версию дерева (для бинарных путей — entry без данных или с пустым data).
- `restore` заменяется на общий механизм "дозаполнения" данных по списку путей.

Преимущество: fs API становится универсальным "файловым деревом". Бинарная политика — забота persist + kind.

**Решение №2:**  
Оставить `Binary` вариант внутри, но убрать специальные методы из публичного API. Persist получает доступ через `pub(crate)` или отдельный internal-модуль. Внешний API видит только общие `visit_entries` + `read_bytes`. Это компромисс (меньше изменений в Node).

**Решение №3:**  
Сделать содержимое файла всегда `enum Content { Text(String), Binary(Vec<u8>) }` внутри `Node::File`, но скрыть это за общими методами. Специальные visit_* убрать. Решение близко к текущему, но чуть чище.

## Сравнительная таблица предлагаемых решений

| Пункт | Решение | Плюсы | Минусы | Сложность миграции | Соответствие KISS/YAGNI |
|-------|---------|-------|--------|--------------------|-------------------------|
| 1 (неиспользуемый API) | Удалить file_ext, split_path, сделать Node pub(crate) | Прямо решает задачу, меньше публичной поверхности | — | Низкая (только api.rs) | Отлично |
| 2 (убрать Vfs/VfsFs) | Решение №1 (Node + free fn + with_fs) | Максимально близко к желаемому в ТЗ ("только Node + mkdir/create_file...") | Потребуется переписать много вызовов (state, install и др.), переделать сигналы | Средняя-высокая | Хорошо (убирает лишнюю обёртку Vfs) |
| 2 | Решение №2 (opaque + переместить VfsFs) | Меньше ломающих изменений в fs | Всё равно придётся именовать типы в некоторых местах | Низкая-средняя | Средне |
| 3 (бинарные) | Решение №1 (Node = File+bytes, общий visit, бинарность в persist+kind) | Полностью убирает специфику из ядра fs, универсальный API | Persist усложнится (нужно самому решать stripping при snapshot) | Средняя | Отлично (YAGNI: fs не должен знать про IDB/LS split) |
| 3 | Решение №2 (скрыть через pub(crate)) | Минимальные изменения в Node | Не полностью решает "не предоставлять специальные методы" | Низкая | Средне |

(Для пункта 1 другие решения не нужны — достаточно одного прямого.)

## Ваше заключение

...