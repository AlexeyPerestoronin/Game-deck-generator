# result

- `run_invoke` = `...` (TODO) → запускает `invoke {command}` через `subprocess` в `self._cwd` (нужен рабочий invoke-tool для агента)
- `_tools` без `run_invoke` → tool зарегистрирован с параметром `command` (иначе агент не увидит метод)
- `tool_help.md` без секции → добавлена справка `run_invoke` (NOTE в DefaultTools требует актуализировать help)
- тестов нет → `tests/default_tools_run_invoke_test.py` (проверка регистрации, cwd, декодирования, таймаута)
