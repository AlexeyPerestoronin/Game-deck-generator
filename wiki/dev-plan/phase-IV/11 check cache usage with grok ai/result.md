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
