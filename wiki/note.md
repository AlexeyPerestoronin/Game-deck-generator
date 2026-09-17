# Note — remove_default_game (2026-09-17)

## Выполненные доработки
1. Удалён `default_game` + поддержка/загрузка `games/conf.json5`.
2. CLI: вместо `--name` (с относительными именами) — обязательный `--game <GAME>` + опциональный `--deck <DECK>`.
3. catalog упрощён (без default_game логики).
4. Работа без games/conf.json5 подтверждена (в т.ч. через root conf путь).
5. cargo check + тесты deck_gen — зелёные. Ручные запуски list/html с новыми флагами — работают.
6. Обновлены релевантные доки/README (этап-2).

## Что проверено автоматически (этап-1 + рефакторинг)
- cargo check -p deck_gen --features cli + --no-default-features — OK.
- cargo test -p deck_gen --features cli — 5/5 OK.
- Изменения только в deck_gen (минимальный scope по плану). deck_gen_wasm не затронут (API prepare_* сохранены).

## Требуемые действия от пользователя (обязательно)
1. Удалить файл `games/conf.json5` (он больше не требуется и не читается; можно `git rm`).
2. Пересобрать `deck_gen.exe` (если используешь pre-built из корня): `cargo build --release -p deck_gen --features cli` + скопировать, или запустить `start.bat`.
3. Обновить свои скрипты/вызовы CLI:
   - старое: `deck_gen list foo` или `deck_gen html --name foo` (опиралось на default_game)
   - новое: `deck_gen list --game new-game --deck foo`  (или без --deck чтобы все колоды игры)
   - Для monopoly (id="monopoly", папка=monopoly-2.0): `--game monopoly`
4. Если есть внешние ссылки на поведение "default game" или games/conf.json5 — поправить.
5. (опционально) Проверить, что в твоём окружении `deck_gen list --game <id>` и генерация работают как раньше, только с явным --game.

После удаления games/conf.json5 и пересборки — задача полностью закрыта.
