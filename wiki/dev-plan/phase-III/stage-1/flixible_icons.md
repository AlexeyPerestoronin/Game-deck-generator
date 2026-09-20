# Задача
Стили иконок кнопок Activity Bar задаются файлами в репозитории: правки PNG (из draw.io) меняют вид кнопок без правки Rust-SVG.

## Текущее поведение (факт из кода)
- Иконки кнопок — инлайн SVG 16×16, `fill="currentColor"`, в `deck_gen_wasm/ui/src/icons/activity.rs`.
- Кнопки: `SaveButton` (если ещё не удалена задачей `remove_save_button.md`), `DownloadButton`, `NewGameButton`, `LoadGameButton`, `PrepareHtmlButton`, `PreparePdfButton`, `SplitPreviewButton`, `ClearButton`.
- Состояния сейчас цветом CSS (`.activity-btn` / `:hover` / `:focus-visible` / `.active`), не отдельными картинками. `:active` (зажатие) отдельно не оформлен.
- Статика уже копируется Trunk: favicon через `<link data-trunk rel="copy-dir" href="favicon" />` в `deck_gen_wasm/index.html`.
- Кнопки Explorer `NewFile` / `NewFolder` — текстовые `.text-btn`, не иконки Activity Bar. Их не трогать.

## Каталог файлов
Корень: `deck_gen_wasm/icons/buttons/` (не `deck_gen_wasm/buttons/`).

На каждую кнопку Activity Bar — подпапка с именем модуля кнопки и ровно три файла:

```
deck_gen_wasm/icons/buttons/
    download/
        off.drawio.png
        on.drawio.png
        click.drawio.png
    load_game/
        off.drawio.png
        on.drawio.png
        click.drawio.png
    new_game/
        …
    prepare_html/
        …
    prepare_pdf/
        …
    split_preview/
        …
    clear/
        …
    save/          # только если кнопка Save ещё есть в Activity Bar
        …
```

Смысл файлов:
- `off.drawio.png` — без наведения;
- `on.drawio.png` — hover / keyboard focus;
- `click.drawio.png` — pointer `:active` (кнопка зажата).

Для `split_preview` класс `.activity-btn.active` (режим split включён) оставить как сейчас рамкой/цветом слева; четвёртый PNG не вводить. В `.active` показывать `on.drawio.png`.

## Размер
- Все PNG **одинакового** пиксельного размера.
- Достаточный минимум под кнопку 40×40: **32×32** CSS-пикселя логического размера 16×16 (как нынешний SVG), либо отображение 32×32 в кнопке, если так лучше читается — но тогда все кнопки одинаково.
- Фон PNG прозрачный, читаемый на `--activity: #333333`.
- Мотив близок к текущим глифам (диск/стрелка/папка/PDF/HTML/split/корзина), чтобы не менять смысл кнопок.

## Код
1. Скопировать каталог `icons` в dist так же, как favicon: в `deck_gen_wasm/index.html` — `<link data-trunk rel="copy-dir" href="icons" />`.
2. В корневом `Trunk.toml` `[watch]` добавить `deck_gen_wasm/icons` (сейчас watch только `src` / `style.css` / `index.html` / `Cargo.toml`).
3. Заменить SVG в `icons/activity.rs` на `<img>` (или CSS `background-image`) с путями вида `icons/buttons/<id>/off.drawio.png`. Три кадра переключать CSS (off по умолчанию, on на `:hover`/`:focus-visible`, click на `:active`). Не оставлять мёртвый SVG «на всякий случай».
4. `currentColor` на PNG не действует — не красить PNG через `color` кнопки. Hover/click = смена файла, не фильтр.
5. `aria-hidden` на картинках: доступное имя по-прежнему у `<button aria-label=…>`.

## Не делать
- Не менять обработчики кликов, тултипы, ConfirmModal.
- Не переносить файловые иконки Explorer (`ui/src/icons/files.rs`) в PNG — это другая задача.
- Не вводить тему/локализацию.
- Не подключать новые npm/JS-бандлы.

## Приёмка
- В Dist после `trunk build` лежат PNG; в UI кнопки показывают `off`, на hover — `on`, на mousedown — `click`.
- Правка любого `off.drawio.png` + пересборка меняет вид кнопки без правки `.rs`.
- Для Split preview должна быть уникальная иконка на активацию (вместо подсвечивания) `active.drawio.png`.
- Все кадры одного размера; ни одна кнопка Activity Bar не осталась на инлайн-SVG.

***

Имена подпапок совпадают с модулями в `deck_gen_wasm/ui/src/buttons/`.

# Особые указания
1. Изменения в коде должны быть минимальными!
2. Запрещено менять существующую архитектуру!
3. Какой код использовать для анализа:
   - крейт `deck_gen_wasm_ui`:
     - `deck_gen_wasm/ui/src/icons/activity.rs`
     - `deck_gen_wasm/ui/src/icons/mod.rs`
     - `deck_gen_wasm/ui/src/buttons/*.rs` — только разметка иконки внутри кнопки (не логика клика)
     - `deck_gen_wasm/style.css` (`.activity-btn`, `:hover`, `:focus-visible`, `.active`)
   - оболочка приложения (не отдельный крейт):
     - `deck_gen_wasm/index.html` (copy-dir, как у `favicon`)
     - `Trunk.toml` в корне репозитория (`[watch]`)
     - `deck_gen_wasm/favicon/` — только как образец copy-dir + суффикс `.drawio.png`
   - новые файлы: `deck_gen_wasm/icons/buttons/<button>/{off,on,click}.drawio.png`
   - прочие файлы и папки репозитория ИГНОРИРУЙ;
3. Краткий отчёт (50-100 строк) о проделанной работе в формате `было→стало(почему)` напиши в `wiki\result.md`.
4. Если какие-то пост-действия требуются от меня напиши их в `wiki\note.md`.

# Дополнительные указания

## Как убедиться в правильности решения:
1. крейты, код которых подвергался изменениям, должны проходить сборку и проверку локальными unit-тестами;
   - `cargo test -p deck_gen_wasm_ui --lib`
   - `cargo check -p deck_gen_wasm`
   - `trunk build` — в `deck_gen_wasm/dist/icons/buttons/` есть PNG

## Как правильно писать код:
1. этап-1: решить поставленную задачу и убедиться в её работоспособнои требуемым образом;
   - код необходимо писать простой, понятный и прямолинейный, без сложных абстракций;
   - если логика кода сложная, но позволяет написать простые unit-тесты для проверки, их надо написать;
2. этап-2: когда поставленная задача будет решена, код, созданный и зафиксированный на этапе-1, необходимо отрефакторить согласно правилам записанным в `wiki\prompts\refactoring-rules.md`
   - при организации кода придерживайся стиля той кодовой базы в которую вносишь изменения;

## Бережливый подход
1. Максимально береги баланс токенов:
   - использовать X-Search ЗАПРЕЩЕНО;
   - использовать Web-Search ЗАПРЕЩЕНО:
     - если необходимо, сформулируй в чате запрос на разрешение поиска с описаем какую информацию хочешь найти и для чего она нужна в рамках решаемой задачи;
