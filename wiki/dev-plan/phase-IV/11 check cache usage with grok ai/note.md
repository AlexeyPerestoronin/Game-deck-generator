Проверка решения (форматирование `WiKi/templates`) выполнена в этой сессии.

Проверь `Sessions statistic` именно этого прогона:
- 1-й request: `cached tokens = 0` — нормально, префикса ещё нет;
- со 2-го request: `cached tokens > 0` и растёт вместе с history.

Если после нескольких итераций всё ещё 0 — пришли usage одного response (repr/`MessageToDict`), сверим имя поля в установленной версии SDK.
