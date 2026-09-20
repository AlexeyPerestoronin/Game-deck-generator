# Требуются действия

1. Установить SDK: `pip install google-genai`
2. Создать файл `API_KEY_GEMINI` в корне репозитория (аналог `API_KEY_XAI`) с ключом Gemini API
3. Проверить, что id модели `gemini-3.8-flash` доступен для этого ключа; если нет — подставить актуальный id в `Tools/harness/agents/gemini.py`
4. Запуск: `inv harness.run-gemini-agent --prompt "..."`
5. `WiKi/rules/refactoring.md` прочитать не удалось (вне allowed_dirs этой сессии) — рефакторинг сделан по стилю `grok.py`
6. Сборку/unit-тесты запустить не удалось (в сессии нет shell). Python-крейты harness тестами не покрыты; Rust-крейты не менялись
7. В корне репозитория есть untracked `gemini.py` — прочитать/трогать его нельзя (вне allowed_dirs); если это черновик, его стоит убрать или перенести вручную
