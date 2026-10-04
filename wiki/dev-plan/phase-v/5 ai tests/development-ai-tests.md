# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача на разработку: AI live-тесты в отдельном crate и настройки AI с GitHub
Вынести ручные live-тесты AI из `deck_gen_wasm_ai` в отдельный crate `deck_gen_wasm_ai_tests`, убрать bundled-файлы AI из WASM-сборки, при старте приложения забирать содержимое `Templates/ai-settings` из GitHub `Deck-Games` в VFS, а промпты агента читать из `help/` (их можно править перед `run_loop`).

Зачем: ключ модели нельзя светить в CI/WASM; конфиги и промпты должны жить в репозитории шаблонов, а не в бинарнике; ручной прогон тестов должен подтверждать тот же протокол, что и браузер.

Ожидаемый результат:
- crate `projects/deck_gen_wasm/modules/ai_tests` с `#[ignore]` live-тестами; `deck_gen_wasm` от него не зависит;
- в WASM нет `include_str` на `*.json5` / `ai-help-*.md` / `create-game-pt-*.md` / `edit-game-pt-*.md`;
- старт приложения копирует файлы из GitHub `Templates/ai-settings` в VFS (json5 → `ai-models/`, md → `help/`), не затирая правки пользователя;
- `run_loop` берёт `create-game-pt` / `edit-game-pt` из `help/` по локали UI;
- live-тест Gemini 3.1 Flash-Lite проходит с адекватными ответами (не HTTP 400); в коммите ключ не зашит, ввод через stdin.

## Функциональные требования

### 1. Новый crate только для ручных тестов
- Путь: `projects/deck_gen_wasm/modules/ai_tests`.
- Имя пакета: `deck_gen_wasm_ai_tests`.
- Добавить в `members` workspace `projects/Cargo.toml`.
- **Не** добавлять зависимость на этот crate в `deck_gen_wasm`, `deck_gen_wasm_ai`, `deck_gen_wasm_workspace`, `deck_gen_wasm_ui` и прочие WASM-пакеты.
- Удалить `mod live` и `src/live.rs` из `deck_gen_wasm_ai`.
- `ureq` убрать из `[dev-dependencies]` `deck_gen_wasm_ai` (нужен только тестам).

### 2. Фикстуры моделей — только в тестовом crate
Сейчас в `projects/deck_gen_wasm/modules/ai/bundled/` лежат и модели, и md. После задачи:
- `*.json5` моделей переносятся в тестовый crate (например `ai_tests/bundled/`) и **не** компилируются в WASM.
- Актуальный набор на диске (не ориентироваться на устаревшие `include_str` в `live.rs` / `install.rs`):
  - `gemini-2.0-flash.json5`
  - `gemini-3.1-flash-lite.json5`
  - `deepseek-chat.json5`
  - `grok-3-mini.json5`
- В коде сейчас ещё фигурирует несуществующий `gemini-2.0-flash-lite.json5`. Заменить на `gemini-3.1-flash-lite.json5`. Не воссоздавать старый файл.
- md из `ai/bundled/` (`ai-help-*`, `create-game-pt-*`, `edit-game-pt-*`) из crate `ai` убрать: в браузере они приходят с GitHub, не из `include_str`.

### 3. Браузер: установка AI-файлов с GitHub
Источник: репозиторий уже задан в `deck_gen_wasm_conf::catalog` (`AlexeyPerestoronin/Deck-Games`, ветка `master`), папка `Templates/ai-settings`.
https://github.com/AlexeyPerestoronin/Deck-Games/tree/master/Templates/ai-settings

При старте приложения (тот же момент, что сейчас `Workspace::ensure_ai_files`):
1. Получить список blob в `Templates/ai-settings/` (тот же git-tree/raw, что каталог и `new-game`; не дублировать клиент GitHub без нужды).
2. Скачать **всё** содержимое папки, не фильтруя по локали.
3. Разложить в VFS:
   - `*.json5` → `ai-models/<имя файла>`
   - `*.md` → `help/<имя файла>`
   - это покрывает `<model>.json5`, `ai-help-<en|ru>.md`, `create-game-pt-<en|ru>.md`, `edit-game-pt-<en|ru>.md`.
4. Писать файл только если его нет или тело — HTML-оболочка приложения (`<!doctype html` / `<html`), как сейчас в `install.rs` / `help.rs`. Пользовательский ключ в json5 и правки md **не** перезаписывать.
5. Каталоги `ai-models/` и `ai-models/log/` создавать как сейчас.
6. Если GitHub недоступен, а нужные файлы в VFS уже есть — не падать, работать с тем что есть. Если файлов нет и скачать не удалось — показать ошибку через существующий `status`/`warning`, приложение не ронять.

`needs_ai_install` / `install_ai_defaults` сейчас синхронные и читают `include_str`. После задачи установка сетевая → `ensure_ai_files` (и установка) должны быть async. Вызов в `ui/src/app.rs` уже внутри `spawn_local`.

Константу пути `Templates/ai-settings` и пути промптов (`help/create-game-pt-<locale>.md`, `help/edit-game-pt-<locale>.md`) положить в `deck_gen_wasm_conf`, рядом с `ai::help` / `help::file`.

Логику установки оставить в `ai` или перенести к прочим GitHub-установкам в `template`. Не плодить второй GitHub-клиент.

Юнит-тесты установки — без сети: чистый mapping remote path → VFS path, правила «нет / HTML-shell / не трогать правку». Тесты `install.rs`, которые ждут конкретные bundled json5 в VFS, переписать. Тесты `engine.rs`, которые зовут `install_ai_defaults` ради пустого json5, пусть кладут json5 в VFS сами.

### 4. Промпты агента из `help/` в LFS/VFS
Сейчас `engine.rs` берёт `create-game-pt-*.md` / `edit-game-pt-*.md` через `include_str` и функцию `bundled_prompt_template`.

Нужно:
- читать шаблон из VFS: `help/create-game-pt-<locale>.md` или `help/edit-game-pt-<locale>.md` (локаль UI, как сейчас);
- подстановки `{game_help}`, `{user_prompt}`, `{tools}`, `{file}`, `{game}` не менять;
- пользователь может править эти md в дереве **до** запуска агента — в `run_loop` уходит актуальный текст из VFS, не копия из бинарника;
- нет файла или он HTML-shell → понятная ошибка `run_loop`, без скрытого fallback на `include_str`.

Юнит-тесты заполнения шаблонов: класть md в VFS, не опираться на compile-time константы.

### 5. Live-тесты
Сценарий тот же: 3 вопроса, ожидается живой ответ, не ошибка API.

```
Reply with exactly one word: hello
What is 1+1? Reply with a single number.
Name any color in one word.
```

Требования:
- Тесты `#[ignore]`, не ходят в сеть из обычного `cargo test`.
- Запуск (из `projects/`):  
  `cargo test -p deck_gen_wasm_ai_tests -- --ignored --nocapture --test-threads=1`
- Цель проверки этой задачи: **Gemini 3.1 Flash-Lite** (`gemini-3.1-flash-lite.json5`). Остальные модели можно оставить как `#[ignore]`, но битых путей на `gemini-2.0-flash-lite` быть не должно.
- Тест **падает** на HTTP/API ошибке (в том числе 400) и на пустом/ошибочном теле. Сейчас `live.rs` печатает `ERROR` и всё равно green — так нельзя.
- Ответы «адекватные»: непустой текст; для `1+1` в ответе есть `2`. Не требовать дословного «hello» в одно слово — модели часто нарушают формат.
- Успешный прогон должен означать, что **браузерный клиент** с тем же json5 и тем же протоколом тоже сможет вызвать модель. Параллельный упрощённый клиент как сейчас в `live.rs` (свой `ureq` + свой body без общего кода `gemini`/`openai`) в финале неприемлем: он может быть green при red WASM и наоборот. Использовать сборку URL/body и разбор ответа из `deck_gen_wasm_ai`. Нативный HTTP для этого — минимальная правка (`cfg(not(target_arch = "wasm32"))`), без тащить `ureq` в WASM-зависимости.
- Ключ: в **закоммиченном** коде только ввод из stdin (как `read_api_key` сейчас). Пустой ключ — fail.

Проверка на живой модели (обязательна в работе, в git ключа нет):
- Временный ключ только для отладки: `???`.
- На время отладки можно зашить его в код, чтобы не вводить руками.
- Добиться, чтобы Gemini 3.1 Flash-Lite отвечала по делу, а не `HTTP 400` / error JSON. Если 400 — чинить запрос/конфиг под реальный API этой модели, тем же протоколом, что браузер (не «упростить тест пока не пройдёт»).
- Когда тесты стабильны — **убрать ключ из кода**, вернуть stdin. В diff ключа быть не должно.

### 6. Документация
Обновить абзац про AI в `projects/deck_gen_wasm/README.md`: модели и md больше не bundled в WASM; старт тянет `Templates/ai-settings`; промпты — `help/create-game-pt-*` и `help/edit-game-pt-*`.

## Scope
Минимальный набор (не раздувать):

- `projects/Cargo.toml` — member `ai_tests`
- `projects/deck_gen_wasm/modules/ai_tests/` — новый crate (Cargo.toml, src, bundled json5)
- `projects/deck_gen_wasm/modules/ai/Cargo.toml` — убрать `ureq`, больше нет bundled-ассетов
- `projects/deck_gen_wasm/modules/ai/src/lib.rs` — убрать `live`
- `projects/deck_gen_wasm/modules/ai/src/live.rs` — удалить
- `projects/deck_gen_wasm/modules/ai/src/install.rs` — GitHub вместо `include_str`
- `projects/deck_gen_wasm/modules/ai/src/engine.rs` — промпты из VFS
- `projects/deck_gen_wasm/modules/ai/src/http.rs` — только если нужен нативный POST для live-тестов
- `projects/deck_gen_wasm/modules/ai/bundled/` — опустошить/удалить
- `projects/deck_gen_wasm/modules/conf/src/lib.rs` — `Templates/ai-settings`, пути pt
- `projects/deck_gen_wasm/modules/workspace/src/ai.rs` — async установка
- `projects/deck_gen_wasm/modules/ui/src/app.rs` — `await` установки
- `projects/deck_gen_wasm/modules/template/src/github.rs` (и/или `lib.rs`) — только reuse fetch
- `projects/deck_gen_wasm/README.md`

Не трогать каталог игр, user-help/game-help bundling, UI модалки ключа, сам `run_loop`/tools, кроме чтения шаблона из VFS.

## Проверка корректности решения
1. `cargo test -p deck_gen_wasm_ai` — зелёный без `--ignored`; live-тестов в этом пакете нет.
2. `cargo test -p deck_gen_wasm_ai_tests` без `--ignored` — сеть не дергается (ignore-тесты не бегут).
3. С ключом (временно в коде, потом stdin):  
   `cargo test -p deck_gen_wasm_ai_tests -- --ignored --nocapture --test-threads=1 live_qa_gemini_3_1`  
   три ответа адекватные, тест red при 400.
4. В финальном diff нет API-ключа; ключ снова из stdin.
5. Поиск по `deck_gen_wasm` (кроме `ai_tests`): нет `include_str` на бывшие bundled AI-файлы; `deck_gen_wasm` не зависит от `deck_gen_wasm_ai_tests`.
6. Ручная сборка WASM не обязана открывать браузер в этой задаче, но код старта должен компилироваться: `ensure_ai_files` async и вызывается из уже существующего `spawn_local` в `App`.

# Дополнительные указания
1. Экономь токены: читай минимум необходимого; правки вноси через patch.
2. Если что-то не получается со второго раза, или ты понимаешь, что контекст задачи сильно разрастается относительно цели задачи в минимальном воплощении, или что-то какая-то информация не дана, но является важной для правильной реализации поставленной задачи → не фантазируй и не додумывай за меня → остановись и задай вопрос → я подскажу и направлю.
3. Результаты — в `result.md` рядом с `todo.md`, формат `было→стало(почему)`.
4. Действия с моей стороны — в `note.md` рядом с `todo.md`.
