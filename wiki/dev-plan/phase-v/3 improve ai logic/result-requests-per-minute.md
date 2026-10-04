# result-requests-per-minute

- json5 `requests_per_second` → `requests_per_minute` (старое имя не читается; пресеты `1` RPS → `10` RPM, как в задаче)
- `ModelConf.requests_per_second: Option<u32>` / `max_rounds: u32` → `requests_per_minute: Option<u32>`, `max_rounds: Option<u32>` (`None` = без лимита: RPM omit/−1; `max_rounds: -1`; omit max_rounds → 12)
- `requests_per_minute: 0` фильтровался как «нет лимита» → локализованная ошибка EN+RU (0 не unlimited)
- `http::throttle`: пауза `1000/RPS` только со 2-го раунда → gap `60000/N` мс **и** скользящее окно 60 с (не больше N вызовов); первый вызов сразу, если квота есть; троттлится только `complete`
- `run_loop` `for 1..=max_rounds` → цикл без скрытого потолка при `max_rounds == None`
- bundled `ai-help-{en,ru}` таблица optional fields и примеры — RPM / `-1` / `0`
- `Cargo.toml` комментарий delay: RPS → RPM
- native-тесты: парсинг RPM/max_rounds/−1/omit/старое имя/0; `interval_ms(2) ≥ 30000`; квота N=1 с одной отметкой в окне → нельзя
- боевые Q&A (не запускались): `projects/deck_gen_wasm/modules/ai/src/live.rs`, `#[ignore]`, 4 конфига, общие 3 вопроса, ключ из stdin, ответ в консоль

Как гонять live (из `projects/`):

```
cargo test -p deck_gen_wasm_ai -- --ignored --nocapture --test-threads=1 live_qa_gemini_2_0_flash
cargo test -p deck_gen_wasm_ai -- --ignored --nocapture --test-threads=1 live_qa_gemini_2_0_flash_lite
cargo test -p deck_gen_wasm_ai -- --ignored --nocapture --test-threads=1 live_qa_deepseek_chat
cargo test -p deck_gen_wasm_ai -- --ignored --nocapture --test-threads=1 live_qa_grok_3_mini
```

Тест спросит API-ключ в консоли, прочитает bundled json5, сделает 3 вопроса. Содержание ответов не проверяется.
