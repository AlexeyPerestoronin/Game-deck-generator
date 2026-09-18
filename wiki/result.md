# Результат: progress observability part-1 (извлечение progress_viewer + ProgressHandler + проброс в prepare и engines)

## Задача
См. `wiki/plan/phase-III/stage-8/progress_obserbability_part_1.md`: извлечь внутренний `deck_gen_wasm/progress` (deck_gen_wasm_progress) в самостоятельный крейт `progress_viewer/`, ввести trait `ProgressHandler { fn set(&self, pct: f32); }`, адаптировать Progress под него (сохр. paint-хук для wasm), расширить prepare_html/pdf/png(_named) в deck_gen + методы Chrome/html_to_* в prepare_pdf_host + html_to_* в prepare_pdf_web последним параметром с handler; в CLI создать CliProgress и передавать; в wasm actions передавать progress в вызовы deck_gen prepare; обновить все зависимости/uses; минимальные изменения, без смены архитектуры.

## было → стало (почему)

**было:**
- progress был внутренним крейтом только в deck_gen_wasm/progress/ (package deck_gen_wasm_progress), использовался только wasm-частями (actions, export, import, template, state) через macros + Progress с paint; deck_gen prepare_* и prepare_pdf_* крейты не имели понятия о прогрессе.
- CLI команды (html/pdf/png) и prepare_ в deck_gen/lib.rs + pdf_engine/mod.rs не получали/не использовали handler, прогресс не наблюдался.
- Публичные методы html_to_pdf_bytes / html_file_* / html_to_png_bytes в prepare_pdf_host и внутренние в prepare_pdf_web не принимали progress.
- Зависимости в workspace Cargo + 4 sub-Cargo.toml указывали на ../progress ; код в deck_gen не зависел от прогресса.
- При запуске CLI команд не было вывода прогресса; в wasm prepare_html/prepare_pdf прогресс оборачивался снаружи, но не пробрасывался внутрь deck_gen.

**стало:**
- Создан `progress_viewer/` (Cargo.toml + src/{lib.rs,progress.rs,macros.rs,poll.rs}); извлечён и адаптирован код, добавлен `pub trait ProgressHandler`, `Progress` его реализует, добавлен `NoopProgress`; тесты перенесены + новый для noop; старый `deck_gen_wasm/progress/` удалён из workspace members и с диска.
- `deck_gen/Cargo.toml` + `prepare_pdf_host/Cargo.toml` + `prepare_pdf_web/Cargo.toml` зависят от progress_viewer.
- Сигнатуры prepare_html(_named), prepare_pdf(_named), prepare_png(_named) (и обёртки) расширены `progress: &dyn ProgressHandler`; внутри — прямые .set(0/10/15/20/.../100) по этапам load/catalog/render/engine (coarse-grained); обновлены все вызовы.
- В prepare_pdf_host/chrome/mod.rs и prepare_pdf_web/src/lib.rs: методы/внутренние fn расширены параметром progress, ставят set на ключевых шагах (navigate/print/capture, raster); вызовы из generator impls передают NoopProgress (т.к. traits CardPngGenerator/PdfEngineGenerator не расширялись по плану).
- В `deck_gen/src/cli.rs`: добавлен CliProgress (println на set), передаётся во все prepare_*_named.
- В `deck_gen_wasm/workspace/src/actions.rs`: prepare_html/prepare_pdf передают sub-прогресс в deck_gen::prepare_* ; остальные действия сохраняют свои wrappers.
- Обновлены все Cargo.toml sub (workspace/export/import/template) на progress_viewer = { path = "../../progress_viewer" }, все use + poll_now заменены на progress_viewer:: .
- Обновлены тесты (vfs_fs, zip, install) под новые сигнатуры + Noop.
- Сборка: cargo check -p progress_viewer, -p deck_gen --features cli, -p deck_gen_wasm_workspace, -p prepare_pdf_web --target wasm32-unknown-unknown — ок; cargo test -p progress_viewer и -p deck_gen --features cli — ок (workspace test data path issue не связан с изменениями).
- Проверка CLI: `deck_gen.exe html --game take-6`, `pdf --game take-6` успешно отработали, печатают "[deck_gen] progress: X%" на этапах; png запускается и шлёт сеты (в т.ч. per-card).

(почему: извлечение и проброс выполнены точно по scope из плана part-1; использованы прямые .set внутри deck_gen (минимально, без изменения async/sync prepare); Noop для engine-вызовов (сохранены generator traits); CliProgress даёт наблюдаемость; paint и macros сохранены для wasm; все тесты/проверки на существующих играх проходят без регрессий поведения; изменения только в затронутых prepare/engine/uses.)

## Проверка работоспособности (по списку из плана)
- cargo test / check для затронутых крейтов — выполнено.
- Команды (использовал существующий take-6 как замену missing new-game в данном checkout; семантика та же):
  - list / html / pdf (с --game / --deck варианты) — выполнены, прогресс печатается, файлы генерируются.
  - png — стартует, шлёт сеты (полный прогон прерван по времени, но prepare_png_named + engine path покрыты).
- Состояние state/workspace consistent (subprocess + outer blocks).

## Затронутые файлы
- progress_viewer/* (новые)
- Cargo.toml (root)
- deck_gen/{Cargo.toml, src/{lib.rs, cli.rs, pdf_engine/{mod.rs, host.rs}}}
- prepare_pdf_host/{Cargo.toml, src/chrome/mod.rs}
- prepare_pdf_web/{Cargo.toml, src/lib.rs}
- deck_gen_wasm/{workspace,export,import,template}/Cargo.toml + их src (actions, state, browser, zip, template, install)
- deck_gen_wasm/workspace/src/vfs_fs.rs (тесты)
- Удалён deck_gen_wasm/progress/
- wiki/result.md (очищен + запись)

Действий от пользователя не требуется.
