# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача на разработку: Локализация help-файлов (`help/` в VFS)
Локализовать `ai-help`, `game-help`, `user-help` (en/ru). В VFS все help лежат в корневой папке `help`. При смене локали догружать только отсутствующий файл текущей локали; уже загруженные файлы других локалей не удалять.

Зачем: help должен быть на языке UI; агент читает game-help той же локали.

Ожидаемый результат: bundled источники `*-en.md` / `*-ru.md`; в VFS пути вида `help/<name>-<en|ru>.md`. Старт и смена локали (сейчас полный reload) ставят файл текущей локали, если его нет или это HTML-shell. Пользовательские правки не затирать.

## Функциональные требования

### Имена и места
Локализованные **исходники** (include_str в crate):

| было | стало |
|---|---|
| `modules/ai/bundled/ai-help.md` | `ai-help-en.md` + `ai-help-ru.md` (там же в `bundled`, json5 моделей не переносить) |
| `modules/template/user-help.md` | `user-help-en.md` + `user-help-ru.md` |
| `modules/template/game-help.md` | `game-help-en.md` + `game-help-ru.md` |

В **VFS** (локальная ФС рабочей области) все help — в корневой папке `help/`:

- `help/ai-help-en.md`, `help/ai-help-ru.md`
- `help/user-help-en.md`, `help/user-help-ru.md`
- `help/game-help-en.md`, `help/game-help-ru.md`

Старые пути `user-help.md`, `game-help.md`, `ai-models/ai-help.md` больше не инсталлировать. Миграцию/удаление старых файлов не делать.

EN: текущий английский текст (поправить внутренние упоминания путей, если они ещё указывают на корень / `ai-models/ai-help.md`). RU: полноценный перевод того же содержания, не пустышка и не копия EN.

### Загрузка
- Нужен файл **текущей** UI-локали (`en`/`ru`). Нет файла или тело — HTML-shell приложения → скопировать bundled. Иначе не трогать.
- Файлы другой локали, если уже есть, **оставить**.
- Отдельной live-логики на клик Locale не нужно: сейчас Locale делает reload, при старте `App` снова `ensure_*`. Достаточно чтобы `ensure_user_help` / `ensure_ai_files` (и game-help) смотрели на текущую локаль и ставили только недостающий файл.
- Если файла локали нет в bundled (будущий язык) — не выдумывать; для en/ru оба файла обязательны.
- Превью help при пустых вкладках открывает **текущий** `help/user-help-<locale>.md`.

### Кто читает
- Агент (`system_prompt` или шаблон) читает game-help **текущей локали** из VFS (`help/game-help-<locale>.md`), не старый `game-help.md`.
- Константы путей — в `deck_gen_wasm_conf` (сейчас `help::PATH`, `game_help::PATH`, `ai::HELP`). Сделать locale-aware хелперы, не размазывать `"help/user-help-en.md"` по UI.

## Scope
- `projects/deck_gen_wasm/modules/conf/src/lib.rs` — пути `help/`
- `projects/deck_gen_wasm/modules/ai/src/install.rs` + bundled `ai-help-*.md` (удалить старый `ai-help.md` после замены)
- `projects/deck_gen_wasm/modules/template/src/help.rs`, `game_help.rs` + файлы help рядом с crate
- `projects/deck_gen_wasm/modules/workspace/src/actions.rs` — `ensure_user_help` / preview path
- `projects/deck_gen_wasm/modules/workspace/src/ai.rs` — `ensure_ai_files` / game-help install
- `projects/deck_gen_wasm/modules/ai/src/engine.rs` — **только** путь чтения game-help (не выносить промпты в md)
- тесты install: нет файла локали → копия; другая локаль уже есть → не удаляется; HTML-shell → reinstall; пользовательский текст → не затирать

Не трогать модалку AI, throttle, 503, `Workspace::clear` (кроме того что help после clear должен по-прежнему жить вне `games/` — это уже так, если help в `help/`).

## Проверка корректности решения

**Тесты native**
- `needs_*` / `install_*` для en при пустом VFS → появляется только en (или оба — нет: **только текущая** локаль).
- После install en поставить ru-локаль в тесте и install → ru появился, en остался.
- HTML-shell текущего файла → переустановка; кастомный markdown → нет.

**Ручной смоук**
1. Чистый старт EN: в дереве `help/user-help-en.md` (preview), `help/game-help-en.md`, `help/ai-help-en.md`. Нет обязательного ru, пока не переключали.
2. Переключить RU (reload) → появились `*-ru.md`, en-файлы если были — на месте.
3. Агент Create/Edit использует содержимое `help/game-help-ru.md` при RU (видно в первом запросе/логе).

# Дополнительные указания
1. Экономь токены: читай минимум необходимого; правки вноси через patch.
2. Если что-то не получается со второго раза, или ты понимаешь, что контекст задачи сильно разрастается относительно цели задачи в минимальном воплощении, или что-то какая-то информация не дана, но является важной для правильной реализации поставленной задачи → не фантазируй и не додумывай за меня → остановись и задай вопрос → я подскажу и направлю.
3. Результаты — в `result-help-locale.md` рядом с этой задачей, формат `было→стало(почему)`.
4. Действия с моей стороны — в `note-help-locale.md` рядом с этой задачей.
