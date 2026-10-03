# note

Действия с вашей стороны:

1. Вписать API-ключ в `ai-models/gemini-2.0-flash.json5` (или другую модель) в VFS. Предустановки с пустым ключом.
2. Для DeepSeek/Grok из браузера задать `proxy_url` (CORS `blocked`). Без прокси будет понятная ошибка, не паника.
3. Заменить PNG кнопки `ai edit`: сейчас стоят кадры `prepare_html`. Нужны `addons/icons/buttons/ai_edit/{off,on,click}.drawio.png`, после этого поменять `src` в `AiEditIcon`.
4. Поправить текст `game-help.md` / `ai-help.md` если формулировки не те.
5. Ручной смоук после WASM-сборки (чистый старт, пустой ключ, Gemini create, ai edit, autogen disabled, DeepSeek без proxy).

Не делал (намеренно): чат, diff/confirm, ключи в localStorage, Settings, PNG/PDF у агента, качать README с GitHub.
