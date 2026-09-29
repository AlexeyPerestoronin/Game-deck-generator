# Заключение
Вариант 2: логика кэша сессии для stateless-агента верна, подсчёт `cached tokens` был неверный.

# было→стало(почему)

**Кэш сессии (логика отправки сообщений) — без изменений.**

было: полный history копится в `self.__chat`, каждый `sample()` уносит tools + все сообщения, `x-grok-conv-id` и `conversation_id` стабильны на сессию.
стало: то же самое.
почему: для stateless xAI это и есть prefix-cache. Префикс следующего запроса = промпт предыдущего, tools не меняются, sticky-id не сбрасывается. Сообщения в кэш попадают; чинить сессию не нужно.

**Подсчёт cached tokens — исправлен.**

было: `response.usage.prompt_tokens_details.cached_tokens` (схема OpenAI REST).
стало: `response.usage.cached_prompt_text_tokens` (поле `SamplingUsage` в xai_sdk / gRPC).
почему: агент ходит в официальный `xai_sdk` (`chat_pb2`), а не в OpenAI-compatible HTTP. У protobuf-usage нет `prompt_tokens_details`, `getattr(..., None)` всегда давал `None`, счётчик оставался 0 даже при реальных cache hit.

# Проверка решения: шаблоны `WiKi/templates`

**development.md**

было: роль + задача + 4 указания; нет scope, DoD и способа проверки.
стало: добавлены Scope, Критерии готовности, Проверка решения; указания унифицированы.
почему: агенту нужны границы чтения и явный Definition of Done; шаблон остаётся коротким.

**planning.md**

было: две конкретные задачи phase-III слиты в один файл — это не шаблон.
стало: универсальный шаблон планирования (цели корректировки, блоки `<...>`, scope, результат).
почему: `make_task_template` копирует файл как `todo.md`; конкретный текст phase-III нельзя переиспользовать.

**refactoring.md**

было: битая ссылка `wiki\prompts\refactoring-rules.md`, плейсхолдеры `???`, отчёт смешан с указаниями.
стало: пути `WiKi/harness/rules/refactoring-*.md`, секции целей/scope/режима/отчёта, единые указания.
почему: агент должен читать реальные rules; формат отчёта с «Ваше заключение» сохранён.
