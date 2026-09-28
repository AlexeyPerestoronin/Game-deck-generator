Нужно прогнать любую задачу на Grok с несколькими итерациями (tool-loop) и в `Sessions statistic` проверить `cached tokens`.

Ожидание:
- 1-й request: `cached tokens = 0` — нормально, префикса ещё нет;
- со 2-го request: `cached tokens > 0` и растёт вместе с history.

Если после нескольких итераций всё ещё 0 — пришли usage одного response (repr/`MessageToDict`), сверим имя поля в установленной версии SDK.
