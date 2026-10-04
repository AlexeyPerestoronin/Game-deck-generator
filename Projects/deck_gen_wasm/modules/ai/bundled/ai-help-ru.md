# Конфиги моделей AI

Кладите по одному JSON5-файлу на модель в `ai-models/<id>.json5`. Имя файла без `.json5` должно совпадать с полем `id`. Новые файлы появляются в выборе модели без пересборки приложения.

Логи каждого запуска пишутся в `ai-models/log/<дата-время>-<id-модели>.md` (Markdown, открывайте в Предпросмотре).

## Обязательные поля

| Поле | Смысл |
|---|---|
| `id` | Идентификатор модели, который уходит в API (и основа имени файла) |
| `label` | Имя в списке выбора |
| `kind` | `"gemini"` или `"openai-compat"` |
| `base_url` | Корень API (завершающий слэш не обязателен) |
| `auth` | `"query-key"` (`?key=`) или `"bearer"` (`Authorization: Bearer`) |
| `api_key` | Секрет. Оставьте пустым, пока не вставите ключ. Пустой ключ не даст запустить запрос. |
| `cors` | `"browser"`, если API разрешает запросы со страницы; `"blocked"`, если нет |

## Необязательные поля

| Поле | Смысл |
|---|---|
| `proxy_url` | **Префикс** CORS-прокси. К нему дописывается настоящий URL API. Пример: `https://corsproxy.io/?` |
| `api-key-hosting` | URL страницы поставщика, где создают API-ключ. Имя поля берите в кавычки (`"api-key-hosting"`) из‑за дефиса. Если задано, в модалке AI появляется кнопка **Запросить API-ключ**, которая открывает этот URL. |
| `requests_per_minute` | Потолок HTTP-вызовов к модели за 60 с и пауза не короче `60000 / N` мс. Опустите поле или `-1` — без троттлинга. `0` недопустим. |
| `max_rounds` | Раунды вызова инструментов, после которых цикл останавливается. Если поле опущено — значение по умолчанию в коде. `-1` — без потолка раундов. |

Лишние поля игнорируются.

Если `cors` равен `"blocked"` и `proxy_url` пуст, модель нельзя использовать из браузера.

## Пример Gemini

```json5
{
  id: "gemini-2.0-flash",
  label: "Gemini 2.0 Flash",
  kind: "gemini",
  base_url: "https://generativelanguage.googleapis.com/v1beta",
  auth: "query-key",
  api_key: "YOUR_KEY",
  cors: "browser",
  proxy_url: "",
  requests_per_minute: 10,
  max_rounds: 12
}
```

URL Gemini: `{base_url}/models/{id}:generateContent?key=...`

## Пример OpenAI-совместимого API (DeepSeek, Grok и другие)

```json5
{
  id: "deepseek-chat",
  label: "DeepSeek Chat",
  kind: "openai-compat",
  base_url: "https://api.deepseek.com",
  auth: "bearer",
  api_key: "YOUR_KEY",
  cors: "blocked",
  proxy_url: "https://corsproxy.io/?",
  requests_per_minute: 10,
  max_rounds: 12
}
```

URL запроса: `{base_url}/chat/completions` (если `base_url` ещё не заканчивается этим путём).

Grok (`https://api.x.ai/v1`) использует тот же `kind`.

## Ключи

Сервера нет, ключа на этапе компиляции тоже нет. Вставьте ключ в `api_key` в этом файле или в поле API-ключа в модалке Create/Edit AI (это значение действует только на запуск и **не** записывается обратно сюда). Файл остаётся в рабочей области браузера (Сохранить / local storage). Не коммитьте настоящие ключи.
