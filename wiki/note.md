# Note — remove_game_prefix_from_deck (2026-09-17, доработка)

Задача доработана полностью:
- Убрано требование совпадения имени колоды с именем папки.
- Убрана вся логика префикса игры в name колоды (backward-compat checks удалены).
- Имя колоды = значение `name` из data.json5 (любое, уникальное в игре).
- Все указанные команды pdf/html/list --game ... --deck ... (с quoted/unquoted, cyrillic) работают.
- cargo test -p deck_gen + build: OK.
- Только правки в deck_gen/src.

Действий от пользователя не требуется. Всё готово.
