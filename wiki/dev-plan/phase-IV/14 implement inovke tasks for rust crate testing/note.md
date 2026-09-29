Прогони из корня репозитория (у агента не было cwd/PATH для invoke):

- `invoke tools.run-deck-gen-tests`
- `invoke tools.run-deck-gen-tests --name <test>`
- `invoke tools.run-deck-gen-wasm-tests`
- `invoke tools.run-deck-gen-wasm-tests --name <test>`

Если имена пакетов в `projects/Cargo.toml` отличаются от `deck_gen` / `deck_gen_wasm` — поправь константы `_DECK_GEN_CRATE` / `_DECK_GEN_WASM_CRATE`.
