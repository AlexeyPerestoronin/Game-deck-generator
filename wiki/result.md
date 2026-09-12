# Copy / past в explorer (`deck_gen_wasm`)

## Explorer: выбор

- было: один путь в `selected`; клик всегда заменял выделение и для папки ещё раскрывал её → стало: рядом `multi_selected: HashSet<String>`; обычный клик по-прежнему один элемент, Ctrl+клик добавляет/снимает путь без открытия файла и без toggle папки (нужен множественный выбор из ТЗ).
- было: визуально подсвечивалась только строка `selected` → стало: строка выглядит выбранной, если она сама или любой предок в `multi_selected` (выбор папки красится как в VS Code, включая видимых детей).
- было: ПКМ всегда вызывал `select` и сбрасывал набор → стало: ПКМ по уже выбранному элементу набор не трогает, иначе выделяет только его (способ №2: Ctrl-набор → ПКМ → `copy`).

## Explorer: метка copy

- было: буфера копирования не было → стало: `copy_planned` — набор путей, без OS clipboard (ТЗ: не копировать в промежуточный буфер, только пометить).
- было: нет отличия «запланирован» → стало: класс `copy-planned` (синяя полоска слева) на строке дерева.
- было: пункт меню без состояния → стало: `MenuState.copy_marked`; пункт `copy` с классом `copy-marked` слегка синий, пока путь в плане.
- было: снять метку было нельзя → стало: повторный `copy` на одном элементе снимает метку; после успешного `past` план очищается.

## Контекстное меню

- было: файл — Rename/Delete (+Preview) → стало: плюс `copy`.
- было: папка — load file(s)/Rename/Delete → стало: плюс `copy` и `past` (лейблы как в ТЗ, не Paste).
- было: команды только delete/rename/preview/load_files → стало: `copy` зовёт `mark_copy`, `past` — `paste_into`.

## Алгоритм copy/past

- было: нечего копировать внутри VFS → стало: `Vfs::copy_entries_into` клонирует узлы (текст и binary) в целевую папку.
- было: конфликт имён падал бы с already exists → стало: `unique_name` (`file`, `file-1`, …), исходник не удаляется.
- было: папка+файл внутри неё вставились бы дважды → стало: `top_level_paths` выкидывает потомков, если предок тоже в плане.
- было: вставка в себя могла бы читать дерево во время мутации → стало: сначала clone всех источников, потом insert (вставка папки в себя даёт вложенную копию).
- было: способ №1 не копился → стало: каждый `copy` toggle’ит один путь в плане, затем `past` в папку.
- было: способ №2 не копился → стало: если ПКМ-цель входит в multi-select (>1), `copy` помечает весь набор.

## Workspace-сигналы

- было: `selected` один, session его пишет → стало: `selected` остаётся primary (create file/folder, вкладки, persist); multi-select и copy-plan в session не кладём (эфемерны).
- было: delete/rename/clear забывали только selected/expanded/tabs → стало: те же операции чистят/переписывают `multi_selected` и `copy_planned`.
- было: `selected.set` размазан по actions → стало: `set_primary_selection` синхронно ставит primary и одноэлементный multi.

## Файлы

- было: логика выбора только в `tree.rs` + `Workspace::select` → стало: чистые функции в `workspace/copy_plan.rs` (тесты без DOM).
- было: VFS умел mkdir/create/remove/rename → стало: `clone_node` + `insert_node` + `copy_entries_into` рядом с остальными мутациями.
- было: CSS только `.tree-row.selected` → стало: `.copy-planned`, `.selected.copy-planned`, `.context-item.copy-marked`; `user-select: none` на строках (Ctrl+клик не выделяет текст).
- было: web-sys без MouseEvent → стало: feature `MouseEvent` для `ctrl_key()`.

## Тесты

- было:  нет тестов copy/select-set → стало: VFS (файл, папка, binary, суффикс, ошибка, вставка в себя) + copy_plan (toggle, multi-mark, nested drop, визуальный предок) + меню содержит `copy`/`past`.
- проверка: `cargo test -p deck_gen_wasm --offline` — 43 ok.

## Рефакторинг (этап-2)

- было: всё в handlers UI → стало: план копирования отдельно от VFS и от Leptos; UI только красит и шлёт команды (KISS, один модуль — одна ответственность).
- было: без шапок → стало: комментарии модулей `copy_plan`, `commands`, explorer/tree, menus, VFS; публичные API с rustdoc.
