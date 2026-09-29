# result

`tools/__init__.py`: TODO-заглушки `run_deck_gen_tests`/`run_deck_gen_wasm_tests`, задачи не зарегистрированы в collection → `cargo test --manifest-path projects/Cargo.toml -p deck_gen|deck_gen_wasm` (опциональный фильтр `--name`), общий `_run_crate_tests`, задачи в collection (нужен прогон unit-тестов по крейту; DRY; иначе invoke их не видит).
