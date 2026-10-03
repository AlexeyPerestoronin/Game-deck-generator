# AI model configs

Put one JSON5 file per model in this folder (`ai-models/<id>.json5`). The file name without `.json5` should match the `id` field. New files show up in the model selector without rebuilding the app.

Logs of each run are written to `ai-models/log/<date-time>-<model-id>.md` (Markdown, open in Preview).

## Required fields

| Field | Meaning |
|---|---|
| `id` | Model id sent to the API (and the file stem) |
| `label` | Name in the selector |
| `kind` | `"gemini"` or `"openai-compat"` |
| `base_url` | API root (no trailing slash needed) |
| `auth` | `"query-key"` (`?key=`) or `"bearer"` (`Authorization: Bearer`) |
| `api_key` | Your secret. Leave empty until you paste a key. Empty key refuses to run. |
| `cors` | `"browser"` if the API allows requests from a web page; `"blocked"` if not |

## Optional fields

| Field | Meaning |
|---|---|
| `proxy_url` | CORS proxy **prefix**. The real API URL is appended. Example: `https://corsproxy.io/?` |
| `requests_per_second` | Pause between HTTP calls. Omit for no throttle. |
| `max_rounds` | Tool-calling rounds before the loop stops. Default in code if omitted. |

Extra fields are ignored.

If `cors` is `"blocked"` and `proxy_url` is empty, the model cannot be used from the browser.

## Gemini example

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
  requests_per_second: 1,
  max_rounds: 12
}
```

Gemini URL: `{base_url}/models/{id}:generateContent?key=...`

## OpenAI-compatible example (DeepSeek, Grok, others)

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
  requests_per_second: 1,
  max_rounds: 12
}
```

Request URL: `{base_url}/chat/completions` (unless `base_url` already ends with that path).

Grok (`https://api.x.ai/v1`) uses the same `kind`.

## Keys

There is no server and no compile-time key. Paste the key into `api_key` in this file. The file stays in the browser workspace (Save / local storage). Do not commit real keys.
