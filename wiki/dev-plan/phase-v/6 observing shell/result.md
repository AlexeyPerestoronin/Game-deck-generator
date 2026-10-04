# result

## tools/harness/agents/tools/default_tools.py
было: `run_shell` / `run_invoke` / `_run_approved_shell` запускали `subprocess.run(..., capture_output=True)` и отдавали весь вывод одним куском после завершения
стало: общий `_run_observed_command` через `Popen` читает stdout+stderr по мере появления, пишет в консоль и возвращает тот же декодированный текст (`utf-8` / `oem` / `cp1251`)
почему: длинные команды выглядели как зависание; git-хелпер `_run_git` оставлен с `capture_output`, потому что его stdout разбирается программно

## tools/harness/agents/tools/tool_help.md
было: у `run_shell` / `run_invoke` / `request_command_shell_execution` не было упоминания живого вывода
стало: добавлена строка, что вывод транслируется в консоль по мере появления
почему: справка должна соответствовать фактическому поведению
