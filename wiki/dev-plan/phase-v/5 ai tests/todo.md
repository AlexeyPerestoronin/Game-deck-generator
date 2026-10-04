# AI live-тесты: план

## Этап 1 — решение
- [x] conf: `Templates/ai-settings`, пути `create-game-pt` / `edit-game-pt`
- [x] crate `deck_gen_wasm_ai_tests` + member workspace; json5 в `ai_tests/bundled`
- [x] убрать live/ureq/bundled из `deck_gen_wasm_ai`
- [x] install: mapping GitHub → VFS, без include_str; тесты без сети
- [x] engine: промпты из `help/` VFS
- [x] http: native POST (`ureq`, не wasm)
- [x] complete_user_text: URL/body/parse из gemini/openai
- [x] template github: reuse fetch для catalog repo
- [x] workspace/ui: async `ensure_ai_files`
- [x] README
- [x] unit-тесты зелёные
- [x] live Gemini 3.1 Flash-Lite (ключ временно в коде → stdin)

## Этап 2 — рефакторинг
- [x] KISS/YAGNI, комментарии модулей, Cargo.toml
