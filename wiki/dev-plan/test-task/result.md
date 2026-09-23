# Результаты

`Tools/harness/agents/tools/default_tools.py`: `get_file_diff` = `...` → `git.Git(dirname).diff('HEAD', '--', filename)` после `_check_access(..., 'r')` (пара к `apply_diff_patch`/`git apply`, без проверки расширения)

`Tools/harness/agents/tools/default_tools.py`: список `_tools` без `get_file_diff` → команда добавлена сразу после `apply_diff_patch` (тот же блок text tools, агент видит инструмент)

`Tools/harness/agents/tools/tool_help.md`: не было секции `get_file_diff` → добавлена после `apply_diff_patch` (NOTE: обновлять справку при изменении списка команд)
