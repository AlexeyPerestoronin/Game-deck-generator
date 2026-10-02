# result

## было → стало (почему)

- Globals: плоский список папок `Game-deck-generator` → каталог `AlexeyPerestoronin/Deck-Games` (`master`, `Games/`, маркер `game.json5`) с карточками Icon/Name/Settings и поиском по кнопке/Enter. Почему: ТЗ §3/§8, стиль VSCode Extensions.
- Local: только кнопка Load → кнопка + реактивный список игр из persist-VFS (`game.json5`), метаданные из `rules/preview/info.json5`, дефолты если файла нет. Почему: ТЗ §2/§6/§7.
- Клик по карточке ничего не делал → открывает html-preview афиши (`TabKind::Preview`); Settings с `stop_propagation` и командами (Local: Prepare HTML/PDF этой игры; Globals: Load Game в persist). Почему: ТЗ §1/§5.
- Preview читал только persist-VFS → persist, иначе `temp_vfs`. Почему: афиши каталога не должны попадать в Explorer/session.
- Нет `temp_vfs` → `Workspace.temp_vfs` (+ catalog_* сигналы), не входит в `snapshot()`/autosave/IDB, чистится в `clear()` и при reload. Почему: ТЗ §4.
- Prepare HTML/PDF гонял весь persist-VFS → обёртка isolate выбранных `{game-root}` → prepare → merge обратно только под этими roots; 0 игр / 0 галочек → warning; 1 игра — сразу; 2+ — hover-меню чекбоксов без DelayedTooltip. Почему: ТЗ §9. `deck_gen` не менялся.
- `(no games listed)` хардкод → locale-ключи (имя-заглушка, Search, пустой каталог, нет совпадений, loading, warning). Почему: ТЗ §11.
- CSS карточек без иконки 42×42 / selected/hover → `.game-row`/`.game-icon`/`.games-search`/`.prepare-game-menu`. Почему: ТЗ §10.
- README/arch: new-game only → фраза про каталог Deck-Games + `temp_vfs`.

## тесты

- `game_roots_from_paths` / `catalog_game_roots`: `Games/a/game.json5` → `Games/a`, вложенность, игнор без маркера.
- `parse_info_json5`: полное; нет name/icon/preview; битый → None.
- `info_matches_query`: EN/RU/tag, регистр, пустой query.
- `isolate_game_trees` / `merge_game_trees`: в working только выбранная игра, `games/conf.json5` не копируется; обратно только файлы под выбранными roots.
- `section_flex` — прежние тесты на месте.
- locale: новые ключи EN+RU в `all_keys_have_en_and_ru`.

`invoke tools.build-deck-gen-wasm` — успех. `invoke tools.run-deck-gen-wasm-tests` — `running 0 tests` (wasm-пакет без wasm-тестов; native unit-тесты в modules этим таском не гоняются).

## ручная проверка

Среду `trunk serve` из этого промпта не запускал (нет shell кроме findstr). Ожидаемое поведение по ТЗ:

1. Games → Local: после Load — карточка с иконкой/именем; Settings → Prepare HTML/PDF этой игры.
2. Клик по Local-карточке → Preview афиши или `default-preview.html`.
3. Globals грузит Deck-Games; Search фильтрует; Load Game кладёт игру в persist → появляется в Local.
4. Reload: persist на месте, `temp_vfs`/каталог сброшены, каталог перечитывается при открытии Games.
5. 2+ игр: hover Prepare HTML — чекбоксы (сняты); 0 отмеченных + клик — warning; одна отмеченная — готовится только она.
6. 1 игра: меню нет, клик готовит её.
7. Explorer не показывает файлы каталога из `temp_vfs`.
8. PNG-команд нет.

## замечания

- `default-preview.html` лежит в `projects/deck_gen_wasm/template/`. Иконка по умолчанию — валидный 1×1 PNG в `DEFAULT_ICON_PNG` (`catalog.rs`); бинарный `default-icon.png` среда записи не умеет положить в git. Подмена файла не нужна для рантайма.
- Ветку каталога не переключал: константа `master`. Если GitHub ответит 404 на tree — ошибка уйдёт в UI Globals, ветка сама не меняется.
- `install_new_game` / `games/conf.json5` для шаблона new-game не трогал; в scoped-prepare conf не копируется.
