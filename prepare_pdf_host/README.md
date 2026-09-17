# prepare-pdf-host

Движок HTML→PDF/PNG для нативного CLI. Ищет Chrome/Chromium/Edge, запускает headless и использует CDP: print_to_pdf для PDF, capture screenshot (clip to card element) для PNG.

Крейт не зависит от `deck_gen`: подключается через `HostPdfEngine` (pdf) и напрямую (png) в сборке с фичей `cli`. В WASM не линкуется.
