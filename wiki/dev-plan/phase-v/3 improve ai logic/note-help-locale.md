# note-help-locale

Ручной смоук (после reload):

1. Чистый старт EN: в дереве `help/user-help-en.md` (preview), `help/game-help-en.md`, `help/ai-help-en.md`. Файлов `*-ru.md` нет, пока не переключали локаль. Старые `user-help.md` / `game-help.md` / `ai-models/ai-help.md` не создаются.
2. Переключить RU (reload) → появились `help/*-ru.md`, en-файлы если были — на месте.
3. Create/Edit AI при RU: в первом запросе/логе содержимое `help/game-help-ru.md`.

Правки пользователя в уже лежащем markdown не должны затираться при reload. HTML-shell текущего файла локали — переустановка.
