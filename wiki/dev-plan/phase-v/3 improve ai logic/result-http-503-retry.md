# result-http-503-retry

- `deck_gen_wasm_conf::ai`: только пути VFS → `HTTP_503_MAX_RETRIES = 20` и `HTTP_503_RETRY_DELAY_MS = 3000` (общий лимит попыток и пауза для всех моделей, не ветка «только Gemini»)
- `ai/src/http.rs` `post_json`: один POST → обёртка: 503 из строки `HTTP {status}` — пауза и тот же POST, пока попыток < max; 409/400/401/сеть без статуса — сразу `Err` (текст как отдаёт `browser::post_json`; Gemini/OpenAI не дублируют retry; throttle не тронут по формуле)
- `browser::post_json`: без нового API → статус по-прежнему в `"HTTP {status}"` (строки достаточно, GitHub GET не ретраится)
- native-тесты: классификация retry vs fail по строке/счётчику, без сети
