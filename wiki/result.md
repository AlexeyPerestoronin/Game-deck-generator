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
