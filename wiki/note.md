# Задача на доработку №2: разбиение `deck_gen_wasm` на wasm-библиотеки

Код не менялся. Ниже — предлагаемое разбиение.

Как это собирается в **один** WASM-модуль (не несколько `.wasm` в браузере):

- каждый подкаталог `deck_gen_wasm/<имя>/` — отдельный Cargo-пакет (`rlib`);
- пакеты добавляются в корневой workspace (`Cargo.toml` → `members`);
- бинарник `deck_gen_wasm` (Trunk / `main.rs`) зависит от них и линкуется в **один** `deck_gen_wasm_bg.wasm`;
- публичная поверхность каждого пакета — только `deck_gen_wasm/<имя>/src/api.rs` (`lib.rs` реэкспортирует `api` и больше ничего наружу не открывает);
- имена пакетов: `deck_gen_wasm_<имя>`, чтобы не столкнуться с общим `fs` / `ui`.

Слои без циклов (стрелка = «зависит от»):

```
conf
 fs → conf
 browser          (js / http / map_join; не знает VFS)
 persist → conf, fs
 export  → conf, fs, browser
 import  → conf, fs, browser
 template → conf, fs, browser
 workspace → conf, fs, persist, export, import, template, deck_gen, prepare_pdf_web
 ui → workspace, fs, conf, browser
 deck_gen_wasm (bin) → ui
```

Не выносить в отдельные крейты: кнопки activity bar (один файл — одна кнопка), `prepare_html` / `prepare_pdf` (разные действия), `html_escape` (два чистых fn, живёт внутри `ui`), `task.rs` (один fn — в `browser`).

---

# Модуль №1: «conf»

Сфера ответственности:
Только неизменяемые compile-time константы (GitHub, шаблон, help-путь, session/IDB ключи, таблицы расширений, лимиты импорта, тайминги UI, имя ZIP). Никакой логики и I/O.

Какой код `deck_gen_wasm` который будет выделен:
- `src/conf.rs`

Внешнее api:
Реэкспорт модулей констант: `github`, `template`, `help`, `session`, `ext`, `import`, `export`, `io`, `ui` (как сейчас). Соседи читают значения, не дублируют таблицы расширений.

---

# Модуль №2: «fs»

Сфера ответственности:
In-memory дерево файлов: пути, тип по расширению, `Vfs` / `Node`, обход без копий байт. **Не** браузер, **не** Leptos, **не** `deck_gen`. Адаптер `VfsFs` (`deck_gen::FileSystem`) остаётся здесь, потому что это форма VFS, а не UI; зависимость `deck_gen` у `fs` допустима и не тянет DOM.

Какой код `deck_gen_wasm` который будет выделен:
- `src/fs/mod.rs`, `path.rs`, `kind.rs`, `vfs_fs.rs`

Внешнее api:
`Vfs`, `Node`, `VfsFs`; пути: `file_ext`, `file_name`, `join_path`, `parent_path`, `path_is_or_under`, `retain_not_under`, `rewrite_prefix`, `rewrite_set`, `split_path`, `unique_name`; `kind` (`FileKind`, `is_image`, `can_highlight`, `explorer_icon`, `syntax_name`, …); `visit_entries` / `visit_binaries` / `without_binaries` / `restore_binaries` как методы `Vfs`.

---

# Модуль №3: «browser»

Сфера ответственности:
Примитивы браузера без знания VFS и UI: JS FFI (`Reflect` / picker / Blob URL), HTTP GET текста, ограниченный `join_all`. Единственное место с `js-sys` / `gloo-net` / `wasm-bindgen` для «сырого» DOM/сети.

Какой код `deck_gen_wasm` который будет выделен:
- `src/js.rs`
- `src/http.rs`
- `src/task.rs`

Внешнее api:
`has_window_fn`, `call0`, `call_async`, `call_async_js`, `blob_url`, `revoke_object_url`; `fetch_text`; `map_join`.

---

# Модуль №4: «persist»

Сфера ответственности:
Снимок сессии: localStorage (текстовое дерево) + IndexedDB (бинарники) + fingerprint. Не ZIP и не picker папки.

Какой код `deck_gen_wasm` который будет выделен:
- `src/persist/mod.rs`, `binaries.rs`

Внешнее api:
`Session`, `from_workspace`, `load_session`, `save_session`, `load_binaries`, `save_binaries`, `save_encoded`, `encode_binaries`, `binaries_fingerprint`.

---

# Модуль №5: «export»

Сфера ответственности:
Упаковка VFS в ZIP и отдача файла пользователю (File System Access или `<a download>`). Не session/IDB.

Какой код `deck_gen_wasm` который будет выделен:
- `src/export/mod.rs`, `zip.rs`, `browser.rs`

Внешнее api:
`vfs_to_zip`, `save_zip_bytes`, `ZIP_FILENAME` (реэкспорт из `conf`).

---

# Модуль №6: «import»

Сфера ответственности:
Выбор локальных файлов/папки, политика расширений/размера, чтение `File`, установка в VFS. Не GitHub и не ZIP.

Какой код `deck_gen_wasm` который будет выделен:
- `src/load_folder/` (`mod`, `input`, `install`, `pick_files`, `picker`, `policy`, `read`)

Внешнее api:
`FileBody`, `PickOutcome`, `PickedFolder`, `PickedFiles`; `pick_and_read_files`, `pick_and_read_folder`; `install_files`, `install_folder`.

---

# Модуль №7: «template»

Сфера ответственности:
Доставка содержимого «снаружи репозитория пользователя»: GitHub tree/raw + установка `new-game` в VFS; bundled `user-help.md`. Оба источника — контент в дерево, не диск-picker и не persist.

Какой код `deck_gen_wasm` который будет выделен:
- `src/github.rs`
- `src/template.rs`
- `src/help.rs`
- файл `user-help.md` (как `include_str` этого крейта)

Внешнее api:
`install_new_game`, `InstalledGame`; `needs_install`, `install_user_help`. HTTP/GitHub listing — внутренние модули, не часть api.

---

# Модуль №8: «workspace»

Сфера ответственности:
Реактивное состояние IDE и сценарии: вкладки, selection, copy/paste, split, draft редактора, `persist` / `prepare_html` / `prepare_pdf` / download / import / new-game. Единственный крейт, который вызывает `deck_gen::prepare_*` и склеивает persist/export/import/template. Не рисует DOM.

Какой код `deck_gen_wasm` который будет выделен:
- `src/workspace/mod.rs`, `actions.rs`, `commands.rs`, `copy_plan.rs`, `split.rs`

Внешнее api:
`Workspace`, `OpenTab`, `TabKind`; методы действий (`persist`, `prepare_html`, `prepare_pdf`, `download`, `load_*`, `add_new_game`, `flush_draft`, `set_draft`, `run_entry_command`, …); `copy_plan::row_looks_selected`; `split::is_previewable` (для контекстного меню).

---

# Модуль №9: «ui»

Сфера ответственности:
Leptos-виджеты: activity bar, explorer, editor (textarea/draft/syntect/preview), меню, модалки, тултипы, иконки. `html_escape` остаётся здесь (нужен только оверлею и `srcdoc`). Не владеет VFS и не вызывает `deck_gen`.

Какой код `deck_gen_wasm` который будет выделен:
- `src/ui/` целиком
- `src/html_escape.rs`
- `src/app.rs` (`App` / `LoadedApp`: restore session + autosave + раскладка)

Внешнее api:
`App` — единственная точка монтирования. Внутренние `Editor`, `Explorer`, `ActivityBar` не обязаны быть публичными за пределами крейта.

---

# Модуль №10: «deck_gen_wasm» (сборочный бинарник, не библиотека)

Сфера ответственности:
Точка входа WASM: panic hook + `mount_to_body(App)`. Trunk, `index.html`, `style.css`, favicon.

Какой код остаётся:
- `src/main.rs`
- `Cargo.toml` бинарника (зависимости = `deck_gen_wasm_ui` + `console_error_panic_hook` / `getrandom`)
- `index.html`, `style.css`, `Trunk.toml`, `favicon/`

Внешнее api:
Нет `api.rs` — это не библиотека.

---

# Сводка по границам

| Крейт | Знает VFS | Знает DOM/Leptos | Знает `deck_gen` |
|---|---|---|---|
| conf | нет | нет | нет |
| fs | да | нет | только `VfsFs` |
| browser | нет | сырой JS, без виджетов | нет |
| persist | да | localStorage/IDB | нет |
| export | да | save picker / `<a>` | нет |
| import | да | File picker | нет |
| template | да | fetch | нет |
| workspace | да | нет (сигналы Leptos) | `prepare_*` |
| ui | через Workspace | да | нет |

`workspace` держит `leptos::RwSignal`, но не компоненты — это сознательный шов: UI рисует, workspace меняет состояние.

---

Ваше решение: 
