# Результат: AI live-тесты

- live-тесты в `deck_gen_wasm_ai` (`mod live` + `ureq` в dev-deps) → crate `deck_gen_wasm_ai_tests` (member workspace, WASM на него не зависит) — ключ не светится в CI/WASM, ручной прогон отдельно
- `ai/bundled/*.json5` в WASM `include_str` → `ai_tests/bundled/` (`gemini-2.0-flash`, `gemini-3.1-flash-lite`, `deepseek-chat`, `grok-3-mini`) — модели только для ручных тестов
- несуществующий `gemini-2.0-flash-lite.json5` → `gemini-3.1-flash-lite.json5` — актуальная модель на диске
- `ai/bundled/*.md` (`ai-help-*`, `create-game-pt-*`, `edit-game-pt-*`) → удалены из crate `ai` — в браузере приходят с GitHub, не из бинарника
- `install_ai_defaults` / `needs_ai_install` (синхронные `include_str`) → `install_ai_files` + mapping `Templates/ai-settings/*.json5`→`ai-models/`, `*.md`→`help/`; писать только если нет файла или HTML-shell — пользовательский ключ и правки md не затираются
- `ensure_ai_files` sync → async: fetch git-tree/raw Deck-Games (`fetch_tree_blob_paths` / `fetch_listed_blobs_for`), `await` в `App` `spawn_local`; при недоступном GitHub работаем с уже лежащими файлами, иначе `status` — не второй GitHub-клиент, приложение не роняем
- `run_loop` брал промпты через `include_str` → читает `help/create-game-pt-<locale>.md` / `help/edit-game-pt-<locale>.md` из VFS; нет файла / HTML-shell → ошибка без fallback — пользователь может править md до запуска
- conf: нет пути GitHub AI и pt → `ai::GITHUB_SETTINGS`, `create_game_pt` / `edit_game_pt` рядом с `ai::help` / `help::file`
- live-клиент (свой ureq + свой body) → `complete_user_text` (URL/body/parse `gemini`/`openai`) + native `ureq` в `http.rs` (`cfg(not(wasm32))`) — тот же протокол, что браузер
- live-тест печатал ERROR и был green → panic на HTTP/пустом теле; для `1+1` в ответе должна быть `2`; пустой ключ — fail
- live Gemini 3.1 Flash-Lite: Hello / 2 / Blue — протокол и json5 рабочие; ключ убран, снова stdin
- README: bundled промпты → старт тянет `Templates/ai-settings`, промпты в `help/create-game-pt-*` и `help/edit-game-pt-*`
