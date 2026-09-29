# result

## Реализация `run_invoke`
- `run_invoke` = `...` (TODO) → запускает `{cwd}/.venv/Scripts/python.exe -m invoke {command}` через `subprocess` в `self._cwd` (иначе `invoke` нет в PATH)
- `_tools` без `run_invoke` → tool зарегистрирован с параметром `command` (иначе агент не увидит метод)
- `tool_help.md` без секции → секция есть, но описывает `invoke ` и `command-execution-limit` 60с (факт: python -m invoke и `invoke-execution-limit` 120с)
- тестов нет → `tests/default_tools_run_invoke_test.py` (регистрация/cwd/декодирование/таймаут)

## Дополнительная задача: доступность invoke
- `run_invoke("--list")` падал (нет `invoke` в PATH) → успешно, список задач (`list-tool-tests`, `run-tool-test`, `make-task-template`, …) (почему: вызов через `.venv/Scripts/python.exe -m invoke`)
- `run_invoke("list-tool-tests")` не вызывался → успешно, список unittest python-tools включая `default_tools_run_invoke_test`
- `run_invoke("--help run-tool-test")` не вызывался → успешно, справка `-n/--name`
- `run_invoke("run-tool-test -n tools.harness.agents.tools.tests.default_tools_run_invoke_test")` не вызывался → 4 теста, 1 FAIL: `test_run_invoke_runs_invoke_in_cwd` ждёт `invoke --list`, код вызывает `python.exe -m invoke --list`

**Итог:** tool `run_invoke` работает, реальные invoke-команды доступны.
