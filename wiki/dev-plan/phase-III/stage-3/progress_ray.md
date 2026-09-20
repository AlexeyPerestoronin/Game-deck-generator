# Задача: «progress ray»

Сделать долгие операции activity bar наблюдаемыми: узкая вертикальная полоска между панелью кнопок и деревом файлов показывает «луч».

## Текущее состояние (из анализа `deck_gen_wasm`)
- Layout `.ide` (`ui/src/app.rs` + `style.css`): CSS Grid `48px var(--explorer-width, 260px) 4px 1fr` — ActivityBar | Explorer | resizer | Editor. Между баром и деревом **нет** колонки.
- Долгие операции живут в `workspace/src/actions.rs` и делят флаг `Workspace.loading` (`state.rs`):
  - `try_begin_async(status)` / `finish_async()` — бинарно (true/false), процента нет.
  - Используют: `load_files_into_folder`, `load_game_from_disk`, `prepare_html`, `prepare_pdf`, `add_new_game`.
  - `download()` статус ставит, но `loading` **не** поднимает.
  - Split Preview и Clear — мгновенные, луч для них не нужен.
- Реальной работы HTML/PDF нет в wasm-обвязке: `deck_gen::prepare_html` / `prepare_pdf` вызываются целиком. Proc-macro крейтов в репозитории нет.
- В WASM UI не перерисуется, пока не отдать кадр (`gloo_timers::TimeoutFuture::new(0).await` уже используется перед prepare_*).

## Визуал
- Виджет на **всю высоту** `.ide`, **между** `ActivityBar` и `Explorer`.
- Ширина: константа `conf::ui::PROGRESS_RAY_WIDTH_PX: u32 = 10`.
- Простой: idle / running, без подписей и кликов.
  - **Idle** (нет операции): полоска целиком **зелёная**.
  - **Running**, прогресс 0: **синяя точка** по вертикальному центру, края пустые (фон панели).
  - **Running**, прогресс p∈(0,100]: синий луч растёт **от центра к верхнему и нижнему краю**; p=100 — полоска целиком синяя.
- Цвета — CSS-переменные (например `--progress-idle` / `--progress-run`), не хардкод в Rust.

## Макросы (обязательные имена, реализуемый синтаксис)

`#[progress_block]` на произвольных statements в stable Rust **нельзя**. Не вводить proc-macro крейт. Сделать **declarative macros** в обычном крейте `deck_gen_wasm/progress` (пакет `deck_gen_wasm_progress`), без зависимости на Leptos/`deck_gen`:

```rust
let mut progress = Progress::new(|pct| { /* записать 0.0..=100.0 в сигнал */ });

progress_wrapper!(progress, {
    progress_block!(progress, 0.0, 10.0, {
        // кусок работы
    });
    progress_loop!(progress, 20.0, 40.0, items, |item| {
        // одна итерация; макрос равномерно распределяет диапазон по len
    });
    progress_block!(progress, 80.0, 100.0, {
        // хвост
    });
});
```

Смысл:
- `progress_wrapper` — в начале `set(0)`, в конце `set(100)`; тело выполняется как есть (в т.ч. `async`, если обёртка стоит внутри уже async-блока).
- `progress_block(from, to, {..})` — `set(from)`, тело, `set(to)`.
- `progress_loop(from, to, iter, |x| {..})` — для i-го элемента `set(from + (to-from)*i/n)` до/после шага.

`Progress` хранит только callback `Fn(f32) + 'static` (или `Rc<dyn Fn(f32)>`). Юнит-тесты на расчёт процентов — в этом крейте, без WASM.

После каждого `set` в **async**-операциях workspace делать `TimeoutFuture::new(0).await`, чтобы луч успел отрисоваться. Это можно спрятать в callback, который ui/workspace передаёт (callback сам spawn/не spawn не должен — достаточно записать сигнал; yield — в actions между блоками).

## Что сделать

### 1. conf
`PROGRESS_RAY_WIDTH_PX = 10` в `conf::ui`.

### 2. крейт `deck_gen_wasm/progress`
`Progress`, три макроса, тесты. Запись в корневой `Cargo.toml` `members` и `Trunk.toml` `[watch]`.

### 3. Workspace
- Поле `pub progress: RwSignal<f32>` (0..=100) рядом с `loading`. Не тащить Progress в persist/Session.
- `try_begin_async`: кроме `loading=true` выставить `progress=0`.
- `finish_async`: `loading=false` (виджет по `loading==false` рисует зелёный idle; значение progress можно оставить).
- Обернуть макросами тела async-операций в `actions.rs` **только на уровне wasm-обвязки**. `deck_gen::prepare_html` / `prepare_pdf` / `install_new_game` — **один** `progress_block` на весь вызов (например 20–90). Не лезть внутрь `deck_gen`, `prepare_pdf_web`, `template`.
- `download()` тоже через `try_begin_async` + блоки (zip encode / save), чтобы луч работал и для Download.
- `load_files_into_folder` не с activity bar, но уже использует `try_begin_async` — обернуть так же, луч общий.

Грубая нарезка (можно чуть сдвинуть числа, сумма логична):
- prepare_html/pdf: yield 0–10, flush+вызов движка 10–90, запись vfs/status 90–100.
- add_new_game: flush 0–10, `install_new_game` 10–90, select/status 90–100.
- load_game_from_disk: pick 0–30, install 30–90, select 90–100.
- download: encode 0–50, save 50–100.

### 4. UI
- Компонент `ProgressRay { workspace }` в `ui/src/bars/progress_ray.rs`, экспорт из `bars/mod.rs`.
- В `LoadedApp` между `<ActivityBar/>` и `<Explorer/>`.
- CSS: колонка `.ide` → `48px var(--progress-ray-width, 10px) var(--explorer-width, 260px) 4px 1fr`. Ширину колонки задать style на `.ide` вместе с `--explorer-width`.
- В формуле max ширины explorer (`app.rs`, сейчас `(ww - 48.0) / 2.0`) вычесть ещё `PROGRESS_RAY_WIDTH_PX`.
- Разметка луча: один контейнер `.progress-ray`; idle — класс/стиль green fill; running — два отрезка от центра (верх/низ) с высотой `p/2`, либо `linear-gradient` / `scaleY`. Без canvas и анимационных библиотек. `pointer-events: none`.

## Не делать
- Не менять `deck_gen`, `prepare_pdf_*`, import picker UI, explorer, editor, кнопки (кроме косвенно `disabled=loading` — уже есть).
- Не вводить proc-macro, async-runtime, web workers.
- Не persist прогресса, не процент в status line.
- Не анимировать Split/Clear.
- Не решать localization/themes/feedback в этом заходе.
- Не переписывать `.ide` с grid на flex.

## Scope кода (анализировать и менять ТОЛЬКО это)
- `Cargo.toml` (только `members`)
- `deck_gen_wasm/Trunk.toml`
- `deck_gen_wasm/conf/src/api.rs`
- `deck_gen_wasm/progress/` (новый крейт)
- `deck_gen_wasm/workspace/Cargo.toml`
- `deck_gen_wasm/workspace/src/lib.rs` (только если нужен re-export — не обязателен)
- `deck_gen_wasm/workspace/src/state.rs`
- `deck_gen_wasm/workspace/src/actions.rs`
- `deck_gen_wasm/ui/src/app.rs`
- `deck_gen_wasm/ui/src/bars/mod.rs`
- `deck_gen_wasm/ui/src/bars/progress_ray.rs` (новый)
- `deck_gen_wasm/style.css` (только `.ide` grid + `.progress-ray*`)
- прочие файлы и папки репозитория ИГНОРИРУЙ (в т.ч. `deck_gen/`, buttons/, explorer, editor, fs, template).

## Приёмка
- В покое между баром и деревом видна узкая (10px) **зелёная** полоска на всю высоту.
- Prepare HTML/PDF, New Game, Load Game, Download: полоска становится синей от центра к краям и к концу операции заполняется; затем снова зелёная.
- Пока операция идёт, прежние кнопки по-прежнему `disabled` через `loading`.
- Explorer по-прежнему ресайзится; max-ширина учитывает новую колонку.
- `cargo test -p deck_gen_wasm_progress -p deck_gen_wasm_workspace` и check ui/conf зелёные.
- Split/Clear не запускают луч.

***

# Задача на доработку №1
Необходимо добавить метод `new_subprocess`, для возможности передачи в качестве параметра с целью отслеживать прогресс изнутри некоторого процесса.
Например:
```rs
    /// Pick local files and copy them into an existing workspace `folder`.
    pub fn load_files_into_folder(&self, folder: &str, warning: RwSignal<Option<String>>) {
        if self.vfs.with(|vfs| !vfs.is_dir(folder)) {
            warning.set(Some(format!("'{folder}' is not a folder")));
            return;
        }
        if !self.try_begin_async("Select file(s)…") {
            return;
        }
        let workspace = *self;
        let folder = folder.to_string();
        spawn_local(async move {
            let progress = workspace.progress_handle();
            progress_wrapper!(progress, {
                let picked = progress_block!(progress, 0.0, 30.0, {
                    workspace.take_pick(warning, pick_and_read_files().await)
                });
                yield_frame().await;
                if let Some(PickedFiles { files }) = picked {
                    let installed = progress_block!(progress, 30.0, 90.0, {
                        workspace.flush_draft();
                        let mut vfs = workspace.vfs.get_untracked();
                        let subprocess: deck_gen_wasm_progress::Progress = progress.new_subprocess(40.0, 90.0);
                        // install_files - заполняет от 0 до 100 шкалу под-прогресса используя те же макросы, которая пропорционально заполняет родительскую шкалу прогресса от 40 до 90 процентов
                        let result = install_files(&mut vfs, &folder, &files, subprocess);
                        (vfs, result)
                    });
                    yield_frame().await;
                    progress_block!(progress, 90.0, 100.0, {
                        match installed {
                            (vfs, Ok(n)) => {
                                workspace.vfs.set(vfs);
                                workspace.expand_ancestors(&folder);
                                workspace.set_primary_selection(Some(folder.clone()));
                                workspace
                                    .status
                                    .set(format!("Loaded {n} file(s) into {folder}"));
                            }
                            (_, Err(err)) => workspace.status.set(err),
                        }
                    });
                }
            });
            workspace.finish_async();
        });
    }
```
Функции `install_files`, `install_folder` и `install_new_game` должны получить дополнительный параметр, который использовать для отслеживания внутреннего прогресса в своих циклах.

# Задача на доработку №2
Необходимо также добавить дополнительный параметр в функцию `save_zip_bytes`, чтобы отслеживать подпрогресс формирования zip архива в зависимости от количества файлов в него добавляемых.

# Задача на доработку №3
Теперь необходимо сделать так, чтобы все actions из workspace (`add_new_game`, `download` и т.д.) запускались в отдельном потоке, чтобы не блокировать работу UI. Важно также, убедиться что любое изменение `pub progress: RwSignal<f32>` в `Workspace` моментально сказывается визуальной составляющей progress-bar.
Разрешаю вносить изменения в дополнительные файлы помимо явно указанных в задаче, если это необходимо, но внимательно следи за тем, чтобы изменения были максимально минимальными и действительно необходимыми.

# Особые указания
1. Изменения в коде должны быть минимальными!
2. Запрещено менять существующую архитектуру!
3. Какой код использовать для анализа: список в секции «Scope кода» выше; прочие файлы и папки репозитория ИГНОРИРУЙ.
4. Краткий отчёт (50-100 строк) о проделанной работе в формате `было→стало(почему)` напиши в `wiki\result.md`.
5. Если какие-то пост-действия требуются от меня напиши их в `wiki\note.md`.

# Дополнительные указания

## Как убедиться в правильности решения
1. крейты, код которых подвергался изменениям, должны проходить сборку и проверку локальными unit-тестами;

## Как правильно писать код
1. этап-1: решить поставленную задачу и убедиться в её работоспособности требуемым образом;
   - код необходимо писать простой, понятный и прямолинейный, без сложных абстракций;
   - если логика кода сложная, но позволяет написать простые unit-тесты для проверки, их надо написать;
2. этап-2: когда поставленная задача будет решена, код, созданный и зафиксированный на этапе-1, необходимо отрефакторить согласно правилам записанным в `wiki\prompts\refactoring-rules.md`
   - при организации кода придерживайся стиля той кодовой базы в которую вносишь изменения;

## Бережливый подход
1. Максимально береги баланс токенов:
   - использовать X-Search ЗАПРЕЩЕНО;
   - использовать Web-Search ЗАПРЕЩЕНО:
     - если необходимо, сформулируй в чате запрос на разрешение поиска с описанием какую информацию хочешь найти и для чего она нужна в рамках решаемой задачи;
