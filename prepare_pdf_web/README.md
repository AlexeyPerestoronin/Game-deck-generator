# prepare-pdf-web

Движок HTML→PDF для браузера. Реализует `deck_gen::pdf_engine::PdfEngineGenerator` и вызывается из `deck_gen_wasm` после `prepare_html`.

Карточный HTML кладётся в скрытый iframe (чтобы отработали CSS и скрипты вёрстки), живой DOM рисуется на canvas, страницы кодируются в JPEG (300 dpi) и собираются в PDF размера карты через `deck_gen`. Рисование из DOM, а не из SVG `foreignObject`, нужно, чтобы canvas в Chromium оставался «чистым».
