# result-ai-modal-api-key

- модалка AI: только prompt + select + Cancel/Run → password-поле ключа слева в `.modal-actions` (глаз показывает/скрывает) и кнопка `API key request` справа от селектора модели, если в json5 есть `api-key-hosting` (ключ можно вставить на один запуск, не правя файл; URL выдачи ключа открывается в новой вкладке)
- `from_conf(vfs, path)` → `from_conf(vfs, path, api_key_override)`; `run_ai` прокидывает строку из поля (непустой override бьёт json5, файл не пишется; оба пустые → та же ошибка `api_key is empty…` через warning)
- `list_model_files`: `(path, label)` → `ModelFile { path, label, api_key_hosting }` через workspace, UI json5 не парсит
- bundled json5: без hosting → `"api-key-hosting"` со страницами Gemini / DeepSeek / Grok (поле опционально; имя с дефисом в JSON5 нужно квотировать)
- `ai-help.md` / locale EN+RU / `.modal-ai*` стили: не было поля и кнопки → документированы и локализованы
