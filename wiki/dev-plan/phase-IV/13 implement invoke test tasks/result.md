list_tool_tests: TODO-заглушка → печать id unittest из `Tools` (`test*.py` и `*_test.py`) (имена сразу пригодны для `run_tool_test`)
run_tool_test: TODO-заглушка → `python -m unittest <name>`, без `--name` запускает все найденные тесты (как в условии задачи)
invoke namespace: только `remove_python_cache` и `make_task_template` → добавлены `list_tool_tests` и `run_tool_test` (иначе `inv` их не видит)
