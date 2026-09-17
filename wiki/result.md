# Результат доработки: remove_game_prefix_from_deck (stage-5)

## Задача
Убрать требование, чтобы `name` колоды в data.json5 содержал префикс игры или совпадал с именем папки расположения. Имя колоды — любое (уникальное в рамках игры), не связано с game-name. Убрать legacy-логику проверки/подстановки префикса. Минимальные изменения, без смены архитектуры. Только крейт deck_gen. Сборка + unit-тесты OK. Проверить командами из "## Как проверить работоспособность".

## было → стало (почему)

**было:**
- dotted_name всегда строил `"{game.id}.{rel from folder path}"` как имя колоды.
- load_deck: после from_manager (который берёт "name" из json) — если != expected (полное), брал rel, и если json name != rel-part → ошибка "name must match relative path".
- В результате: имена колод внутри (Deck.name, labels, output paths derived, queries) всегда с префиксом "game.xxx"; нельзя было использовать json name отличное от имени папки.
- game_for_deck_name / output_dir_for_name / name_matches_query / deck_query — жёстко завязаны на "game." префикс в deck name.
- Доки утверждали "must match", "becomes the prefix of every deck name".
- В примере new-game: "deck-2nd" папка с json "колода №2", "колода 3" с "дополнительная колода" — не работали (падали на prepare).

**стало:**
- declared_name: извлекает `name` из первого `"name": "..."` в raw data.json5 (простой regex, т.к. до expand).
- catalog: LocatedDeck теперь хранит отдельно game_id + name (declared, без префикса); dotted_name переименован/заменён на declared_name (без добавления game.).
- load_deck: убрана вся проверка и force name=expected; имя = то что в json; game_id проставляется из game.
- name_matches_query: обновлён принимать Located, матч по game_id (для --game), по declared name, и по "game.deck" форме (split_once на первом .).
- deck.name теперь короткое (deck-1st / колода №2 / дополнительная колода).
- game lookup в prepare путях: используем deck.game_id (после добавления поля в Deck).
- output_dir_for_name: ослаблено требование префикса (rest = strip или name целиком); model использует его через game_id+name.
- CLI deck_query оставлен без изменений (продолжает генерировать "game" / "game.deck" для query).
- Доки обновлены (модуль, функции, schema).
- Стилистически: простой прямолинейный код (этап-1), затем минимальный рефакторинг (убран synthetic, прямая передача game_id в output_*, убраны неиспользуемые импорты, правки комментов).
- Почему так: минимально (изменения сконцентрированы в catalog + 4 места lookup), без новой арх-ры (те же fn сигнатуры у публичных API catalog/find/prepare, те же flow), deck names теперь decoupled; regex уже был в зависимостях.

**Проверка работоспособности (команды из плана):**
- list --game new-game → deck-1st \n колода №2 \n дополнительная колода
- html/pdf --game new-game   (все 3 колоды)
- html/pdf --game new-game --deck deck-1st
- ... --game "new-game" --deck deck-1st
- ... --game new-game --deck "колода №2"
- ... --game new-game --deck "дополнительная колода"
- ... --game "new-game" --deck "дополнительная колода"
Все отработали без ошибок (render, output dirs с именами колод, pdf с chrome).

cargo test -p deck_gen --features cli : 8/8 passed.
cargo check/build -p deck_gen : OK.
Только правки внутри deck_gen/src (как указано).

## Этапы по инструкции
1. Решение + проверка (прямой код, запуск команд, тесты).
2. Рефакторинг по wiki/prompts/refactoring-rules.md (KISS: нет лишних абстракций; стиль базы сохранён — короткие фактич. комменты, split_once как везде; Cargo.toml не трогал (regex уже использовался); добавил/обновил доки для публичных/модулей где вносил).

## Файлы
deck_gen/src/catalog.rs, model.rs, cli.rs, lib.rs, pdf_engine/mod.rs, render.rs, conf/mod.rs, conf/schema.rs
(минимально по смыслу)
