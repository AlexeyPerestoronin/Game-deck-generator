# note

1. Поправить `test_run_invoke_runs_invoke_in_cwd`: ожидает `invoke --list` и timeout из `command-execution-limit` (30); код запускает `{cwd}/.venv/Scripts/python.exe -m invoke ...` и timeout `invoke-execution-limit` (default 120).
2. Актуализировать `tool_help.md` (`run_invoke`): сейчас `invoke ` + `command-execution-limit` 60с, факт — python venv `-m invoke` и `invoke-execution-limit` 120с.
