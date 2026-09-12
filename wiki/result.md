# Отчёт: tainted canvas при Prepare PDF

## Ошибка
```
Cannot prepare PDF
Failed to execute 'toBlob' on 'HTMLCanvasElement': Tainted canvases may not be exported.
```
Chrome и Edge (Blink). Firefox — без ошибки.

## Где чинили
Кнопка в `deck_gen_wasm` только вызывает `prepare_pdf_web::WebPdfEngine`.
`toBlob` жил в `prepare_pdf_web/src/html_to_jpeg_pages.js`.
Меняли этот JS и комментарии крейта `prepare_pdf_web`.

## Было → стало (почему)

1. Растр карточки: SVG + `<foreignObject>` → `Image` → `canvas.drawImage` → `toBlob`.
   → Растр с уже свёрстанного DOM в iframe (`getBoundingClientRect`, `getComputedStyle`, `fillText`).
   Почему: в Blink SVG-картинка с `<foreignObject>` делает canvas «tainted»; `toBlob` запрещён. Firefox canvas не портит. HTML карточек без внешних картинок — taint даёт сам foreignObject.

2. Стили карточки копировались в SVG через `XMLSerializer` + теги `<style>`.
   → Стили читаются с live-элемента (`getComputedStyle`).
   Почему: отдельная SVG-копия больше не нужна; браузер уже посчитал layout в iframe.

3. Масштаб 300 dpi: `transform: scale()` внутри SVG.
   → `ctx.scale(dpi/96)` на canvas.
   Почему: тот же PRINT_DPI, без SVG.

4. `styleText` собирался из всех `<style>` документа и передавался в растр.
   → Параметр убран.
   Почему: painter не сериализует HTML.

5. Фон карточки: белая заливка canvas + CSS в SVG.
   → Белая заливка canvas + `backgroundColor` каждого узла (прозрачный пропускаем).
   Почему: белая подложка JPEG сохранена; непрозрачные боксы рисуются сами.

6. Рамки: CSS в SVG (в т.ч. только `border-bottom` / `border-right`).
   → Равномерная рамка — `roundRect`+stroke; разные стороны — отдельные линии.
   Почему: шапка и ячейки часто имеют одну сторону рамки.

7. Скругления: CSS `border-radius` в SVG.
   → `borderRadii` + `ctx.roundRect`.
   Почему: `.card` и `.box` со скруглением.

8. Текст: браузер рисовал его внутри foreignObject.
   → По символу: `Range.getBoundingClientRect` + `fillText` (`text-transform` uppercase/lowercase).
   Почему: переносы, выравнивание и letter-spacing уже есть в layout iframe.

9. Списки `<ul><li>`: маркеры CSS в SVG.
   → Диск / круг / квадрат слева от `li`, если `list-style-type !== none`.
   Почему: foreignObject больше не рисует `::marker`.

10. `overflow: hidden` (карточка, блоки текста).
    → `clip` по прямоугольнику/скруглению элемента.
    Почему: длинный текст не должен вылезать за карточку.

11. `<img>` в SVG foreignObject (если появится) тоже травил canvas в Chrome.
    → `drawImage` только для `data:`, `blob:` и same-origin.
    Почему: чужой origin снова даёт taint; в текущих views картинок нет.

12. `iframe srcdoc` + sandbox, `fonts.ready`, `__fitHeaderNames`.
    → Без изменений.
    Почему: вёрстка и подгонка кегля те же, меняется только съём пикселей.

13. `WebPdfEngine` / `html_to_pdf` / `parse_pages` / `pdf_from_jpeg_pages`.
    → Без изменений.
    Почему: ошибка была в экспорте canvas, не в сборке PDF.

14. Кнопка `PreparePdfButton` и `Workspace::prepare_pdf`.
    → Без изменений.
    Почему: текст «Cannot prepare PDF» — заголовок алерта; тело — исключение JS.

15. `prepare_pdf_web/Cargo.toml`: зависимости без пояснений.
    → Короткий комментарий над каждой зависимостью.
    Почему: правило рефакторинга.

16. Шапка `prepare_pdf_web/src/lib.rs`: одна строка.
    → Зачем iframe, зачем paint DOM, зачем JPEG→PDF.
    Почему: правило рефакторинга про комментарий модуля.

## Проверки
- `cargo build -p prepare_pdf_web --target wasm32-unknown-unknown` — ок.
- `cargo test -p prepare_pdf_web` (native) — ок, тестов в крейте нет.
- `cargo test -p prepare_pdf_web --target wasm32-unknown-unknown` — wasm-тест на Windows не запускается (ожидаемо).
- DOM-painter в headless Chrome/Edge отсюда не гонялся.

## Что не трогали
`deck_gen`, `games/`, хостовый Chrome PDF, UI wasm кроме транзитивной зависимости на JS сниппет.
