# Задача: «feedback button»

Добавить кнопку `Feedback` на левую activity bar: по клику открывается почтовый клиент по умолчанию (`mailto:`) с заранее заданными получателем, темой и телом письма.

## Текущее состояние (из анализа `deck_gen_wasm`)
- Левая панель — `ActivityBar` (`ui/src/bars/activity.rs`): 7 кнопок + spacer + `ClearButton` внизу.
- Порядок сейчас: Download, NewGame, LoadGame, PrepareHtml, PreparePdf, SplitPreview, `<div class="activity-spacer">`, Clear.
- Каждая кнопка — отдельный файл в `ui/src/buttons/`, иконка PNG в `ui/src/icons/activity.rs` (кадры `off`/`on`/`click` из `deck_gen_wasm/icons/buttons/<id>/`).
- Тултипы — константы `conf::ui::TOOLTIP_*` в `deck_gen_wasm/conf/src/api.rs`.
- Крейта feedback нет. `mailto:` нигде не формируется. `conf` — только compile-time константы, без форм.
- Паттерн нового крейта: каталог `deck_gen_wasm/<name>/` + запись в корневой `Cargo.toml` `[workspace].members` + `Trunk.toml` `[watch]`.

## Что сделать

### 1. Константы и шаблон письма
В `deck_gen_wasm/conf/src/api.rs` добавить модуль `feedback`:
- `EMAIL: &str = "Alexey.Perestoronin@yandex.ru"`
- `SUBJECT: &str = "Game-Deck-Generator Feedback"`
- тело — `include_str` файла `deck_gen_wasm/conf/forms/feedback-template.md`

Содержимое `feedback-template.md` (короткое, чтобы `mailto:` не упирался в лимит URL):

```md
Game-Deck-Generator feedback

What I was trying to do:


What happened:


Browser / OS:

```

### 2. Новый крейт `deck_gen_wasm/feedback`
Имя пакета: `deck_gen_wasm_feedback`. Без Leptos.

Публичный API:
- `pub fn compose_mailto(email: &str, subject: &str, body: &str) -> String` — чистая сборка `mailto:` с percent-encoding `subject`/`body`. Покрыть unit-тестами (native).
- `pub fn send_feedback_via_email() -> Result<(), String>` — берёт `conf::feedback::*`, вызывает `compose_mailto`, затем `web_sys::window().location().set_href(&url)`. Ошибка, если нет `window`.

Не делать HTML-форму, не слать HTTP, не использовать сторонний SMTP.

### 3. Кнопка в UI
- Новый `ui/src/buttons/feedback.rs` по образцу `download.rs`: `DelayedTooltip` + `<button class="activity-btn">` + иконка, `aria-label="Feedback"`, клик → `send_feedback_via_email()` (ошибку можно игнорировать или записать в `workspace.status` — без нового модала).
- Тултип: `conf::ui::TOOLTIP_FEEDBACK` = `"Send feedback by email."`
- Иконка `FeedbackIcon` в `icons/activity.rs` — тот же PNG-паттерн, что у Download (3 кадра).
- Файлы иконок: `deck_gen_wasm/icons/buttons/feedback/{off,on,click}.drawio.png`. Пока нет оригинала — **скопировать** PNG любой существующей кнопки (например `download/`). В `wiki/note.md` написать, что пользователь заменит рисунки.
- Вставить кнопку в `ActivityBar` **сразу после `Clear`**. Итоговый порядок:
  Download, NewGame, LoadGame, PrepareHtml, PreparePdf, SplitPreview, spacer, Clear, **Feedback** [, Locale/Theme если уже есть из соседних задач — не трогать].
- Подключить крейт в `ui/Cargo.toml`.
- Зарегистрировать пакет в корневом `Cargo.toml` `members` и в `deck_gen_wasm/Trunk.toml` `[watch]`.

## Не делать
- Не менять архитектуру activity bar (не объединять кнопки, не выносить бар в новый виджет).
- Не трогать workspace/fs/import/export/template/persist/`deck_gen`.
- Не менять CSS, кроме случая если без этого кнопка не встаёт (не должно понадобиться — класс `.activity-btn` уже есть).
- Не использовать `dyn Error` в сигнатуре (в кодовой базе ошибки — `String`).

## Scope кода (анализировать и менять ТОЛЬКО это)
- `Cargo.toml` (только `members`)
- `deck_gen_wasm/Trunk.toml`
- `deck_gen_wasm/conf/src/api.rs`
- `deck_gen_wasm/conf/src/lib.rs` (если нужен re-export — сейчас `pub use api::*`, модуль из api подхватится сам)
- `deck_gen_wasm/conf/forms/feedback-template.md` (новый)
- `deck_gen_wasm/feedback/` (новый крейт целиком)
- `deck_gen_wasm/ui/Cargo.toml`
- `deck_gen_wasm/ui/src/buttons/mod.rs`
- `deck_gen_wasm/ui/src/buttons/feedback.rs` (новый)
- `deck_gen_wasm/ui/src/bars/activity.rs`
- `deck_gen_wasm/ui/src/icons/mod.rs`
- `deck_gen_wasm/ui/src/icons/activity.rs`
- `deck_gen_wasm/icons/buttons/feedback/` (новые PNG)
- прочие файлы и папки репозитория ИГНОРИРУЙ.

## Приёмка
- В activity bar есть 8-я кнопка (после Clear). Тултип через 1.5 с: «Send feedback by email.»
- Клик открывает почтовый клиент (или вкладку `mailto:`) с To/Subject/Body из conf+шаблона.
- `cargo test -p deck_gen_wasm_feedback -p deck_gen_wasm_conf` зелёные; `compose_mailto` проверяет encoding и наличие email/subject.
- `cargo check -p deck_gen_wasm_ui` успешен.
- Существующие 7 кнопок без регрессий.

***

# Особые указания
1. Изменения в коде должны быть минимальными!
2. Запрещено менять существующую архитектуру!
3. Какой код использовать для анализа: список в секции «Scope кода» выше; прочие файлы и папки репозитория ИГНОРИРУЙ.
4. Краткий отчёт (50-100 строк) о проделанной работе в формате `было→стало(почему)` напиши в `wiki\result.md`.
5. Если какие-то пост-действия требуются от меня напиши их в `wiki\note.md`.

# Дополнительные указания

## Как убедиться в правильности решения
1. крейты, код которых подвергался изменениям, должны проходить сборку и проверку локальными unit-тестами;

## Как правильно писать код
1. этап-1: решить поставленную задачу и убедиться в её работоспособности требуемым образом;
   - код необходимо писать простой, понятный и прямолинейный, без сложных абстракций;
   - если логика кода сложная, но позволяет написать простые unit-тесты для проверки, их надо написать;
2. этап-2: когда поставленная задача будет решена, код, созданный и зафиксированный на этапе-1, необходимо отрефакторить согласно правилам записанным в `wiki\prompts\refactoring-rules.md`
   - при организации кода придерживайся стиля той кодовой базы в которую вносишь изменения;

## Бережливый подход
1. Максимально береги баланс токенов:
   - использовать X-Search ЗАПРЕЩЕНО;
   - использовать Web-Search ЗАПРЕЩЕНО:
     - если необходимо, сформулируй в чате запрос на разрешение поиска с описанием какую информацию хочешь найти и для чего она нужна в рамках решаемой задачи;
