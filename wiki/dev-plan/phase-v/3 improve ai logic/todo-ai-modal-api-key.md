# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача на разработку: API-ключ в модалке AI (поле ввода + кнопка запроса ключа)
В модальном окне Create/Edit AI добавить: (1) поле ввода API-ключа с приоритетом над json5; (2) кнопку `API key request`, открывающую сайт выдачи ключа выбранной модели.

Зачем: не обязательно править json5, чтобы один раз запустить агента; ключ можно быстро получить у поставщика.

Ожидаемый результат: в `AiPromptModal` слева в ряду Cancel/Run — password-поле ключа (глаз показывает символы); справа от селектора модели — кнопка `API key request`, если в конфиге модели есть `api-key-hosting`. Ключ из поля используется вместо `api_key` файла. В json5 ключ из модалки **не** записывать.

## Функциональные требования

### Поле API-ключа
- Расположение: одна линия с кнопками `Отмена` / `Запустить`, **крайнее слева** (затем Cancel, затем Run).
- Тип как у пароля: символы скрыты; по нажатию на иконку глаза — показать; повторное нажатие — скрыть.
- Пустое поле → использовать `api_key` из json5 выбранной модели.
- Непустое поле → этот ключ **имеет приоритет** над json5. Файл модели не менять.
- Если и поле, и json5 пустые → цикл не стартовать, ошибка как сейчас (`api_key is empty…`) через warning-поток.
- `from_conf` должен принимать override ключа, иначе пустой json5 + ключ в модалке сейчас падает до сети.

### Кнопка `API key request`
- На одной линии **справа** от `<select>` модели.
- Подпись: `API key request` (локализовать EN+RU).
- Клик → новая вкладка браузера (`window.open`, `_blank`) с URL из поля модели `api-key-hosting`.
- Поле **опционально**. Нет ключа / пустая строка → кнопки нет (не disabled-заглушка).
- Смена модели в селекторе сразу показывает/прячет кнопку по выбранному файлу.

### Конфиг модели
В json5 (лишние поля по-прежнему игнорировать):

```text
api-key-hosting: "https://…"   // опционально; URL страницы, где взять ключ
```

В предустановленных bundled json5 проставить URL поставщика:

| файл | url |
|---|---|
| `gemini-2.0-flash.json5` | `https://aistudio.google.com/apikey` |
| `gemini-2.0-flash-lite.json5` | `https://aistudio.google.com/apikey` |
| `deepseek-chat.json5` | `https://platform.deepseek.com/api_keys` |
| `grok-3-mini.json5` | `https://console.x.ai/` |

Парсинг: `ModelConf` + `list_model_files` отдают hosting UI **через workspace**, без зависимости `ui` → `ai`. UI json5 сам не парсит.

Документация поля — в текущем bundled `ai-help` (файл `ai-help.md` или уже `ai-help-en.md`/`ai-help-ru.md`, если локализация help уже сделана). **Не** переносить help в `help/` и **не** переименовывать `requests_per_second` в этой задаче.

### Прочее
- Строки UI — `locale` keys + `dict.json5` + тест полного набора ключей.
- Стили — существующие `.modal*` / `.modal-ai`; глаз и поле вписать в ряд `.modal-actions` без нового «мастера».
- Публичный контракт `AiEngine`: по-прежнему `from_conf` + `run_loop`. Параметр override ключа в `from_conf` (или эквивалент до сети) допустим. Методов `set_api_key` не добавлять.

## Scope
Минимальный набор. Не трогать throttle, 503, help-пути, тексты промптов агента.

- `projects/deck_gen_wasm/modules/ui/src/modals/ai_prompt.rs`
- `projects/deck_gen_wasm/modules/ui/src/windows/games/create_ai.rs`
- `projects/deck_gen_wasm/modules/ui/src/buttons/ai_edit.rs`
- `projects/deck_gen_wasm/style.css` — только ряд модалки / password+глаз / кнопка hosting
- `projects/deck_gen_wasm/modules/locale/src/keys.rs`, `dict.json5`
- `projects/deck_gen_wasm/modules/ai/src/conf.rs` — поле `api-key-hosting`, список моделей для UI
- `projects/deck_gen_wasm/modules/ai/src/engine.rs` — только `from_conf` (override ключа)
- `projects/deck_gen_wasm/modules/workspace/src/ai.rs` — прокинуть override в `run_ai`
- `projects/deck_gen_wasm/modules/ai/bundled/*.json5` — только добавить `api-key-hosting`, прочие ключи не менять
- bundled `ai-help*` — только документ `api-key-hosting`

## Проверка корректности решения

**Тесты (native)**
- Парсинг json5 с `api-key-hosting` и без него (пустое/отсутствующее → нет URL).
- `from_conf`: пустой json5 + непустой override → Ok; оба пустые → Err про api_key; непустой json5 + другой override → в запросе используется override (проверить на `ModelConf`/движке без сети, если ключ хранится в engine).

**Ручной смоук**
1. Модель без `api-key-hosting` → кнопки нет.
2. Gemini → кнопка есть, открывает Studio в новой вкладке.
3. Ключ только в модалке, json5 пустой → Run идёт; json5 не перезаписан.
4. Ключ только в json5, поле пустое → Run как сейчас.
5. Глаз скрывает/показывает символы; поле слева в ряду Cancel/Run.

# Дополнительные указания
1. Экономь токены: читай минимум необходимого; правки вноси через patch.
2. Если что-то не получается со второго раза, или ты понимаешь, что контекст задачи сильно разрастается относительно цели задачи в минимальном воплощении, или что-то какая-то информация не дана, но является важной для правильной реализации поставленной задачи → не фантазируй и не додумывай за меня → остановись и задай вопрос → я подскажу и направлю.
3. Результаты — в `result-ai-modal-api-key.md` рядом с этой задачей, формат `было→стало(почему)`.
4. Действия с моей стороны — в `note-ai-modal-api-key.md` рядом с этой задачей.
