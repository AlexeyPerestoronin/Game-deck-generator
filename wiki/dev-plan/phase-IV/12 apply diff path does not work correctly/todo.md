# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача на исправление: apply_diff_patch doesn't work correct in all cases
Необходимо доработать функцию `apply_diff_patch` (см. tools\harness\agents\tools\default_tools.py), которая выдаёт ошибку `cannot apply diff patch → error: corrupt patch at line 37` при попытке применить патч `.log\2026-09-28 21-59_SpaceXAI_Grok 4.6\tmpu2dmnsl_.patch`.

Кажется, суть проблемы в том, что patch из примера не содержит символ перевода строки в самом конце.
Необходимо учесть возможность применения патчей с этой особенностью (если, конечно, я правильно понял суть ошибки):
- внести соответствующие изменения в `apply_diff_patch`;
- доработать тесты `tools\harness\agents\tools\tests\default_tools_patch_file_test.py`.

## Scope
Читай и меняй только:
- ./tools
Остальное игнорируй.

## Проверка корректности исправления
Должны проходить все тесты `TestPatchFile`

# Дополнительные указания
1. Результаты — в `result.md` рядом с `todo.md`, формат `было→стало(почему)`.
2. Действия с моей стороны — в `note.md` рядом с `todo.md`.
4. Экономь токены: читай минимум необходимого; правки вноси через patch.
