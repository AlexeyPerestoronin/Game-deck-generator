# prepare-pdf-web

Движок HTML→PDF/PNG для браузера. Реализует `deck_gen::PdfEngineGenerator` и `CardPngGenerator`; используется из `deck_gen_wasm`.

HTML одной или нескольких карт в скрытый iframe, DOM на canvas, JPEG (для PDF, 300dpi) или PNG (для per-card, 2x). DOM paint вместо foreignObject — для origin-clean canvas в Chromium.
