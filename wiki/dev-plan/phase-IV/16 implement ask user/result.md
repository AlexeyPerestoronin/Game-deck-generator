# Результат

`tools/harness/agents/tools/default_tools.py`:
- `ask_user` — TODO/`...` → `input(question)` с ошибкой `cannot ask user → ...` (нужен CLI-вопрос и ответ пользователя)
- список `_tools` — без `ask_user` → зарегистрирован с параметром `question` (иначе агент не увидит инструмент)

`tools/harness/agents/tools/tool_help.md`:
- нет раздела `ask_user` → добавлена справка (при изменении списка инструментов нужно обновлять help)
