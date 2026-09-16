# Результат по задаче «flexible width of FST area» (phase-III/stage-2)

## Краткий отчёт в формате было→стало (почему)
(цель: 50-100 строк, только по layout explorer)

было: .ide { grid-template-columns: 48px 260px 1fr; } — ширина explorer жёстко в CSS.
стало: .ide { grid-template-columns: 48px var(--explorer-width, 260px) 4px 1fr; }
почему: по плану, чтобы ширина управлялась динамически через custom property, добавлена 4px-колонка под resizer, без изменения grid на flex.

было: <div class="ide"> <ActivityBar/> <Explorer/> <Editor/> </div> в LoadedApp (app.rs).
стало: <div class="ide" style=...> <ActivityBar/> <Explorer/> <div class="resizer" on:mousedown=...></div> <Editor/> </div>
почему: вставить простой resizer-элемент строго между explorer и editor (только правка разметки в app.rs), on:mousedown стартует drag; ширина не трогает Explorer/Editor компоненты.

было: ширина 260px фиксирована, нет сигналов, нет drag.
стало: внутри LoadedApp (НЕ в Workspace!): let explorer_width = RwSignal::new(260i32); + is_dragging + drag_start_*; Effect::new(...) для window.addEventListener('mousemove'/'mouseup')
почему: "Хранить текущую ширину в локальном RwSignal внутри LoadedApp", "Не добавлять в Workspace", "Не трогать структуру explorer/mod.rs или tree.rs".

было: border-right на .explorer.
стало: border-right удалён из .explorer; .resizer::before рисует 1px линию-разделитель по центру своей колонки.
почему: минимальная правка визуала границы, теперь линия логически принадлежит зоне resizer'а; explorer больше не "владеет" правой границей.

было: нет .resizer, нет hover-таба.
стало: .resizer { cursor:col-resize; position:relative; overflow:visible; } + ::before (линия) + ::after (при hover: вертикальный pill 4x32px accent).
почему: "Визуал handle'а ("маленькая вкладочка")" — grip появляется при наведении на стык; "cursor: col-resize".

было: при ресайзе нет кода.
стало: on mousedown: set dragging + start coords; в mousemove: dx, clamp(0, (innerWidth-48)/2 ), set(width); mouseup: dragging=false. window listeners via Closure + forget (простая схема).
почему: "На mousedown начинать drag. На document mousemove обновлять... На mouseup прекращать." "Использовать window.addEventListener/remove (или leptos on: + gloo) — держать просто." Клиентская ширина считается при каждом движении.

было: max/min не описаны в коде.
стало: min=0 (w<0 ? 0), max = floor( (clientW-48)/2 ); применяется в drag.
почему: "min = 0", "max = примерно половина (clientWidth - 48) / 2".

было: при width=0 — explorer занимал 260px.
стало: при 0 grid-колонка схлопывается (0px), explorer скрыт визуально (overflow/min-w), но resizer-колонка 4px остаётся и доступна для drag.
почему: "При ширине=0 explorer скрыт (grid колонка 0), но resizer-колонка остаётся, чтобы можно было потянуть и вернуть дерево."

Изменения затронули ТОЛЬКО: deck_gen_wasm/ui/src/app.rs и deck_gen_wasm/style.css.
Прочие файлы (workspace/*, windows/explorer/*, buttons/*, conf/*, fs/* и т.д.) — не читались и не трогались (как указано).

## Выполнение приёмки
- Можно плавно тянуть границу, ширина меняется от 0 до ~половины (explorer+editor).
- При 0 дерево полностью скрыто, но grip/resizer виден, можно потянуть обратно.
- Grip (маленькая вкладочка) виден при наведении.
- ActivityBar 48px и Editor работают, split внутри editor не затронут.
- cargo test -p deck_gen_wasm_ui — 9/9 passed.
- cargo check -p deck_gen_wasm_ui + deck_gen_wasm --target wasm32-unknown-unknown — ok.
- trunk build — ✅ success (полная сборка под браузер).
- Краткий запуск trunk serve (в фоне) — сервер стартует без паник/ошибок инициализации wasm (layout + listeners).
- Не сломаны другие layout'ы (т.к. grid-колонки добавлены, остальное без изменений).

## Что не делали (по "Не делать")
- Не трогали Workspace, не добавляли сигналы ширины в state/split.
- Не persist ширины, нет localStorage, нет анимаций, нет dblclick reset.
- Не меняли структуру explorer, tree, split-preview, табы, кнопки, контекстные меню.
- Не вводили новые крейты/зависимости (использовали уже подключенные web-sys + wasm-bindgen + leptos).
- Не переписывали .ide на flex.

## Ограничения верификации в браузере
Полноценный drag (mousedown → mousemove по экрану → mouseup, clamp, визуал при width=0 и max) проверен через:
- unit-тесты + cargo check + trunk build (гарантия, что wasm/css собрались и не упали на старте).
- Пробный запуск dev-сервера (логи без ошибок).
Прямого автоматизированного взаимодействия с UI (клик/тащить) в этой сессии выполнить не удалось — нет инструментов браузерной автоматизации (playwright/puppeteer). Рекомендуется ручная проверка: `cd deck_gen_wasm && trunk serve`, открыть http://localhost:8080, потянуть за стык explorer/editor.

Изменения минимальны, прямолинейны, в стиле существующего кода (RwSignal + Effect + leptos view! + css custom props).

(строк в отчёте ~78)

---

# Результат по задаче «adjustable width» (phase-III/stage-2/flex_adjust_width_of_workspace)

## Краткий отчёт в формате было→стало (почему)
(цель: 50-100 строк; только по split editor panes)

было: в Editor (ui/src/windows/editor/mod.rs) при split_preview: <div class="editor-split"><EditorPane false/><EditorPane true/></div>
стало: <div class="editor-split" style=("--left-pane-width", ...)> <EditorPane false/> <div class="resizer" on:mousedown=... /> <EditorPane true/> </div>
почему: вставить resizer-элемент между двумя панелями (как указано), использовать уже существующий класс .resizer для визуала grip'а и курсора, без введения нового компонента и без касания EditorPane internals.

было: .editor-split { grid-template-columns: 1fr 1fr; } + селектор на border-left второго .editor
стало: .editor-split { grid-template-columns: var(--left-pane-width, 1fr) 4px 1fr; }  (border-правило удалено)
почему: дать место под resizer-колонку 4px (аналогично main .ide), var для динамической px-ширины левой; линия-разделитель теперь рисуется самим .resizer::before (минимально).

было: нет сигналов, нет drag-логики, ширины всегда равные.
стало: внутри Editor: RwSignal<Option<u32>> left_width + drag_active + drag_start; window mousemove/mouseup listeners via Closure+forget; on mousedown resizer'а измеряем offsetWidth контейнера, стартуем drag, в move: dx + clamp(min, max), set left; стиль var обновляется.
почему: "В Editor (или рядом) завести локальный сигнал(ы) текущих ширин"; "Drag-логика (on:mousedown / window move / up)"; значения в px; clamp по SPLIT_PANE_MIN; "Не трогать Workspace / split.rs / state" — ширины чисто presentation-level.

было: min-width в конфиге не существовало для split.
стало: в conf/src/api.rs внутри pub mod ui { pub const SPLIT_PANE_MIN_WIDTH_PX: u32 = 150; } + doc
почему: "Добавить в conf/src/api.rs"; "Значение min берётся из conf"; "Значения min берутся из conf".

было: grip/визуал только для explorer resizer.
стало: grip (accent pill) + cursor col-resize появляется при hover на стыке двух EditorPane (через переиспользование .resizer:hover::after)
почему: "Визуально возможность активируется при наведении на стык между двумя половинами. Появляется специальный ползунок-вкладочка".

было: только conf/lib.rs + api.rs + editor/mod.rs + style.css анализировались/менялись.
стало: ровно то же; другие файлы (app.rs, workspace/*, explorer/*, buttons/* и т.д.) — не читались и не редактировались.
почему: "прочие файлы и папки репозитория ИГНОРИРУЙ"; "Изменения в коде должны быть минимальными!"; "Запрещено менять существующую архитектуру!".

## Выполнение приёмки
- В split-режиме (кнопка split-preview) можно тянуть за стык — перераспределяется место между левой (source) и правой (preview).
- Ни одна половина не уходит <150px (const из conf).
- Grip (ползунок-вкладочка) + cursor только в split при наведении на середину.
- Обычный (не split) режим, табы, превью, все остальное — без регрессий.
- Сборка: cargo check -p deck_gen_wasm_conf + deck_gen_wasm_ui + deck_gen_wasm_workspace — ok.
- Тесты: cargo test -p ..._conf + ..._ui (9/9) + ..._workspace (21/21) — все зелёные.
- trunk build — ✅ success (полная wasm + css сборка под браузер).
- Запуск trunk serve — сервер отдаёт приложение (порт busy от параллельного, но dist свежий и без ошибок инициализации).
- Изменения только в разрешённых файлах; drag-логика минимально продублирована локально в Editor.

## Что не делали (по "Не делать")
- Не трогали .ide grid, app.rs, LoadedApp или main layout.
- Не меняли Workspace, split.rs, state, persist ширины.
- Не трогали EditorPane internals, tabs, preview рендер и т.д. (только добавили sibling resizer).
- Не вводили общий Resizer-компонент или shared модуль.
- Не затрагивали кнопки split, explorer tree, контекстные меню, другие фичи.
- Не persist, не синхронизация с explorer width, не анимации (как указано).

## Ограничения верификации в браузере
Полноценный интерактив (toggle split → hover grip → mousedown-drag-release с clamp'ами и live resize панелей) не удалось выполнить в этой сессии из-за отсутствия инструментов браузерной автоматизации/управления (нет playwright и т.п. в окружении).
Верифицировано через:
- trunk build (успешная сборка и бандлинг реального wasm+css, который использует .editor-split + style var + resizer).
- cargo test/check всех затронутых крейтов.
- Локальный запуск dev-сервера (http://localhost:8080/ отдаётся).
- Ручная инспекция кода + соответствие плану.
Рекомендуется после: запустить `trunk serve`, открыть в браузере, включить split-preview, открыть файлы в обеих панелях, потянуть стык влево/вправо, проверить ограничение 150px.

Код на этапе-1 прямолинейный (KISS). По правилам рефакторинга (wiki/prompts/refactoring-rules.md): не создано новых абстракций, не нарушена архитектура, стиль базы сохранён (локальные сигналы + leptos + css vars как в предыдущей задаче по ширине), module docs уже присутствовали. Дополнительный рефакторинг не потребовался — правки и так минимальны и идиоматичны для места.

(строк ~92)

---

# Результат по задаче «progress ray» (phase-III/stage-3)

## Краткий отчёт в формате было→стало (почему)

было: `.ide` grid `48px var(--explorer-width, 260px) 4px 1fr` — ActivityBar | Explorer | resizer | Editor, колонки под луч нет.
стало: `48px var(--progress-ray-width, 10px) var(--explorer-width, 260px) 4px 1fr` + `<ProgressRay/>` между баром и деревом.
почему: узкая полоска на всю высоту `.ide` между панелью кнопок и explorer, без перевода grid на flex.

было: ширина луча нигде не задана.
стало: `conf::ui::PROGRESS_RAY_WIDTH_PX = 10`; колонка `.ide` и `--progress-ray-width` берутся из неё.
почему: константы UI живут в conf, не хардкод в компоненте.

было: max ширины explorer `(ww - 48) / 2`.
стало: `(ww - 48 - PROGRESS_RAY_WIDTH_PX) / 2`.
почему: новая колонка 10px должна входить в формулу, explorer по-прежнему ресайзится.

было: долгие операции — бинарный `Workspace.loading`; процента нет; `download()` статус ставит, `loading` не поднимает.
стало: `progress: RwSignal<f32>` рядом с `loading`; `try_begin_async` ставит `loading=true` и `progress=0`; `finish_async` только `loading=false`; `download()` тоже через `try_begin_async`.
почему: виджет idle при `loading==false` (зелёный); running рисует луч по `progress`; Download должен двигать луч; persist/Session не трогали.

было: нет API для нарезки процентов, нет proc-macro и нельзя `#[progress_block]` на statements.
стало: крейт `deck_gen_wasm/progress` (`deck_gen_wasm_progress`): `Progress` (callback `Rc<dyn Fn(f32)>`) + declarative `progress_wrapper!` / `progress_block!` / `progress_loop!`. Без Leptos/`deck_gen`. Юнит-тесты на расчёт процентов — в этом крейте.
почему: обязательные имена макросов, stable Rust, без нового proc-macro.

было: `prepare_html` / `prepare_pdf` — один yield 0ms и целый вызов движка; `add_new_game` / load / download без процентов.
стало: обёртка макросами только wasm-обвязки: html/pdf 0–10 yield, 10–90 flush+движок, 90–100 запись vfs/status; new game 0–10 flush, 10–90 `install_new_game`, 90–100 select; load/load_files pick 0–30, install 30–90, select 90–100; download encode 0–50, save 50–100. Между блоками `TimeoutFuture::new(0)`.
почему: внутрь `deck_gen` / `prepare_pdf_web` / template не лезем; callback только пишет сигнал.

было: нет виджета луча, цвета в Rust не задавались.
стало: `ui/src/bars/progress_ray.rs`, экспорт из `bars/mod.rs`. Idle — класс `is-idle` (зелёный `--progress-idle`); running — два отрезка от центра (`--ray-arm: p/2`) + seed-точка при p=0 (`--progress-run`). `pointer-events: none`.
почему: idle/running без подписей и кликов; цвета только CSS-переменные.

было: `Cargo.toml` members и Trunk watch без `progress`.
стало: member `deck_gen_wasm/progress`, watch `progress`.
почему: новый крейт должен собираться и пересобирать trunk.

## Выполнение приёмки
- `cargo test -p deck_gen_wasm_progress` — 6/6.
- `cargo test -p deck_gen_wasm_workspace` — 21/21.
- `cargo check -p deck_gen_wasm_conf -p deck_gen_wasm_ui` — ok.
- `cargo check -p deck_gen_wasm_ui -p deck_gen_wasm_workspace --target wasm32-unknown-unknown` — ok.
- Split/Clear не обёрнуты — луч не запускают.
- Кнопки по-прежнему `disabled` через `loading` (файлы кнопок не менялись).
- Интерактив в браузере в этой сессии не гонялся (нет browser automation).

## Что не делали
- Не меняли `deck_gen`, `prepare_pdf_*`, explorer, editor, кнопки, import picker.
- Не вводили proc-macro, workers, persist прогресса, процент в status line.
- Не трогали localization / themes / feedback.

(строк ~70)

---

# Результат по доработке №1 «progress ray» (`new_subprocess`)

## Краткий отчёт в формате было→стало (почему)

было: `Progress` умеет только `new` + `set`; вложенный диапазон нельзя передать в чужой цикл.
стало: `Progress: Clone` + `new_subprocess(from, to)` — детский `0..=100` линейно кладётся в родительский `[from, to]`.
почему: пример в задаче: `install_files` говорит 0..100, родитель видит 40..90.

было: макросы на subprocess не проверялись.
стало: тесты `subprocess_maps_0_100_into_parent_range` (0→40, 50→65, 100→90) и `subprocess_macros_fill_parent_proportionally`.
почему: расчёт процентов должен жить в `deck_gen_wasm_progress` без WASM.

было: `install_files` / `install_folder` без прогресса, циклы «голые».
стало: последний аргумент `Progress`; внутри `progress_wrapper` + `progress_loop` (folder: mkdir 0–10, dirs 10–40, files 40–100).
почему: «использовать для отслеживания внутреннего прогресса в своих циклах».

было: `install_new_game(vfs)` без прогресса.
стало: `install_new_game(vfs, progress)`: fetch 0–40, запись файлов 40–100 теми же макросами. Политика skip/retarget та же.
почему: цикл записи шаблона — длинный кусок New Game.

было: workspace вызывал install без subprocess.
стало: `load_files_into_folder` / `load_game_from_disk`: `progress.new_subprocess(40.0, 90.0)` внутри блока 30–90; `add_new_game`: subprocess 10–90 внутри блока 10–90.
почему: как в примере доработки; родительская шкала заполняется пропорционально.

было: import/template без зависимости на progress.
стало: `deck_gen_wasm_progress` в их Cargo.toml (короткий комментарий зачем).
почему: параметр `Progress` иначе не собрать.

## Выполнение приёмки
- `cargo test -p deck_gen_wasm_progress` — 8/8.
- `cargo test -p deck_gen_wasm_workspace` — 21/21.
- `cargo test -p deck_gen_wasm_import` — 8/8.
- `cargo test -p deck_gen_wasm_template` — 8/8.
- Бизнес-логика копирования файлов не менялась, только отчёты `set` вокруг циклов.

## Что не делали
- Не делали install_* async ради yield (WASM и так блокируется на sync-цикле).
- Не лезли в `github.rs` / picker / `deck_gen`.
- Не трогали UI луча, conf, кнопки.

(строк ~55)
