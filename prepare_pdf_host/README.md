# prepare-pdf-host

Движок HTML→PDF для нативного CLI. Ищет Chrome/Chromium/Edge (переменные окружения, CDP default, пути из `conf.json5`), запускает headless-браузер и печатает HTML в PDF с размером страницы карты (мм, без полей).

Крейт не зависит от `deck_gen`: его подключает только `HostPdfEngine` в нативной сборке с фичей `cli`. В WASM не линкуется.
