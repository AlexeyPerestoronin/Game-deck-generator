# Нужно с твоей стороны

1. **URL `api-key-hosting` в пресетах** — в задаче стоят типичные страницы ключей:
   - Gemini → `https://aistudio.google.com/apikey`
   - DeepSeek → `https://platform.deepseek.com/api_keys`
   - Grok → `https://console.x.ai/`
   Замени в `todo-ai-modal-api-key.md`, если нужны другие.

2. **«help в корневой папке `help` в LFS»** — в `todo-help-locale.md` это VFS рабочей области: `help/<name>-<en|ru>.md`. Исходники остаются bundled в crate (`include_str`). Если имелся Git LFS / другая папка репозитория — скажи, поправлю задачу.

3. **Переводы** help и промптов в задачах: EN = текущий текст, RU = полный перевод. Готовых RU-файлов нет.

4. **Пауза 503** в плане не задана — в задаче дефолт **3000 мс** + константа в `conf`. Нужна другая — напиши число.

5. Имя `create-game-pt-<locale>.md` взято как в плане (`pt` не расшифровывал).
