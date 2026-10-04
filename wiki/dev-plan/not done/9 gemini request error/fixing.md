# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача на исправление: gemini request error
```
https://generativelanguage.googleapis.com/v1beta/models/gemini-3.1-flash-lite:generateContent?key=???: HTTP 400: {
  "error": {
    "code": 400,
    "message": "Function call is missing a thought_signature in functionCall parts. This is required for tools to work correctly, and missing thought_signature may lead to degraded model performance. Additional data, function call `default_api:ls` , position 2. Please refer to https://ai.google.dev/gemini-api/docs/thought-signatures for more details.",
    "status": "INVALID_ARGUMENT"
  }
}
```

## Scope
Читай и меняй только:
- <пути к файлам и папкам>
Остальное игнорируй.

## Проверка корректности исправления
<как проверить: лог, тест, ручной шаг; либо «не требуется»>

# Дополнительные указания
1. Результаты — в `result.md` рядом с `todo.md`, формат `было→стало(почему)`.
2. Действия с моей стороны — в `note.md` рядом с `todo.md`.
4. Экономь токены: читай минимум необходимого; правки вноси через patch.
