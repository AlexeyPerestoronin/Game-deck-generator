# Задача: «localization»

Добавить переключение языка UI EN ↔ RU (по умолчанию EN) и провести **весь chrome интерфейса** через словарь. Кнопка `Locale` — внизу activity bar, сразу под `Clear`.

## Текущее состояние (из анализа `deck_gen_wasm`)
- Все видимые строки — английские литералы. Крейта locale нет.
- Тултипы семи кнопок: `conf::ui::TOOLTIP_*` (`conf/src/api.rs`).
- Модалки: `ConfirmModal` принимает `'static str` (title/message/confirm_label); `Cancel` захардкожен в `modals/confirm.rs`; `OK` — в `modals/alert.rs`. Тексты Clear/Load Game живут в `bars/activity.rs`.
- Меню: `MenuCommand { label: &'static str }` в `menus/context.rs` (Rename/Delete/Copy/Preview/Past/load file(s)); вкладки — `"Close all"`, `"Close"`, префикс `"Preview "`.
- Explorer: `"Games"`, `"New File"` / `"New Folder"` (title), `"Cannot load files"`.
- Editor empty: `"No preview open."`, `"Select a file to edit…"`, `"Binary file ({} bytes)."`, `"Loading workspace…"`.
- Footer статуса: `workspace.status` — литералы из `workspace/src/actions.rs`, `commands.rs`, `state.rs` (`ask_name("New file name")` и т.п.).
- `index.html`: `<html lang="en">`, `<title>Game deck generator</title>`.
- Смена языка на лету обязательна → `localize()` должен **читать реактивный сигнал** внутри вызова (как `signal.get()` в Leptos), иначе view не обновится.

## Логический модуль `deck_gen_wasm/locale`
Пакет `deck_gen_wasm_locale`. Запись в корневой `Cargo.toml` `members` и `Trunk.toml` `[watch]`.

### API (имена сохранить)
- `pub enum Locale { En, Ru }` — default `En`.
- `pub fn get_active_locale() -> Locale`
- `pub fn change_locale() -> Result<(), String>` — цикл `En → Ru → En`, persist, ok.
- `pub fn localize(key: &str) -> String` — перевод для **текущей** locale; неизвестный ключ → вернуть сам ключ.
- Дополнительно (нужно для тестов и первого кадра): `pub fn set_locale(Locale) -> Result<(), String>`.

Ключи — публичные константы в `locale/src/keys.rs` (часть API), значения констант = строковые id в словаре (`"tooltip-clear"`, не английский текст).

Словарь — `deck_gen_wasm/locale/dict.json5`:

```json5
{
  "tooltip-clear": { EN: "Clear the workspace in this browser.", RU: "Очистить рабочую область в этом браузере." },
  // ...
}
```

Парсить `include_str!` + крейт `json5` (уже есть у `deck_gen` как `"0.4"`) в `OnceLock`. Оба языка обязательны для каждого ключа.

Хранение текущей locale: `RwSignal` в locale-крейте (зависимость `leptos` csr, по образцу workspace). `localize()` делает `.get()` этого сигнала.

Persist: `localStorage` ключ `conf::session::LOCALE_KEY = "deck_gen_wasm.locale"` (`"en"|"ru"`). Не класть в `Session`. Прочитать при первом обращении / старте App.

`html lang` и `document.title` обновлять при смене языка (ключи `html-lang`, `document-title`).

Юнит-тесты (native, без окна — persist тогда no-op или через cfg): каждый `keys::*` есть в dict для EN и RU; `change_locale` циклит два значения; неизвестный ключ возвращается как есть.

## Кнопка Locale
- `ui/src/buttons/locale.rs` по образцу split: два визуальных состояния (EN/RU), класс на кнопке, иконки `icons/buttons/locale/{en,ru}-{off,on,click}.drawio.png`. Нет оригинала — скопировать PNG существующей кнопки. В `wiki/note.md` попросить заменить.
- Клик → `change_locale()`. Тултип тоже через `localize(keys::TOOLTIP_LOCALE)`.
- Место: **сразу под `ClearButton`**. Если уже есть Theme из соседней задачи — Locale **между** Clear и Theme. Не двигать верхние кнопки.

## Какие строки переводить (и только их)

Статический chrome + статус/prompt, которые пишет **сам workspace/ui**. Ошибки из `fs` / `import` / `export` / `template` / `deck_gen` **не** оборачивать в этом заходе (они приходят как `String` с нижних слоёв).

Обязательный набор ключей (имена в `keys.rs` можно чуть унифицировать, но покрытие = этот список):

**conf tooltips (перенести тексты в dict, константы `TOOLTIP_*` в conf удалить, кнопки читают `localize(keys::…)`):**
- семь существующих TOOLTIP_* + тултип Locale.
- если в баре уже есть Feedback/Theme из соседних задач — их тултипы тоже, **не** добавляя сами кнопки.

**activity / buttons / explorer header:**
- aria-label всех activity-кнопок (Clear, Download, …)
- warning titles: Error, Cannot load folder, Cannot prepare HTML/PDF, Cannot load new-game, Cannot load files
- Confirm Clear: title, message, confirm_label
- Confirm Load Game: title, message, confirm_label
- `"Games"`, `"New File"`, `"New Folder"`
- `"Loading workspace…"`

**modals:** `"Cancel"`, `"OK"` — читать через `localize` внутри Confirm/Alert (не прокидывать с каждого вызова).

**menus:** labels Rename, Delete, Copy, Preview, Past, load file(s), Close all; tab `title="Close"`; префикс вкладки Preview.

**editor empty:** оба empty-текста; `"Binary file ({} bytes)."` — `localize` + подстановка числа (`format!` после перевода со `{}` или два куска).

**workspace prompts/status (литералы, не err с VFS):**
- `ask_name`: `"New file name"`, `"New folder name"`, `"New name"`
- `"Could not update workspace"`, `"Nothing to paste"`, `"Unknown command {other}"`
- `"Pasted {n} item(s) into {dest}"`, `"Deleted {path}"`, `"Renamed to {new_path}"`
- `"Workspace cleared"`, `"Opened {path}"`, `"Saved in this browser"`
- `"Select file(s)…"`, `"Select a folder…"`, `"Loading folder…"`, `"Loaded {n} file(s) into {folder}"`, `"Loaded {path}"`
- `"Preparing HTML…"`, `"Prepared HTML for {n} decks"`
- `"Preparing PDF…"`, `"Prepared PDF for {n} decks"`
- `"Loading new-game template…"`, `"Added {path} from {source}"`
- `"Downloading ZIP…"`, `"Downloaded {filename}"`
- `"{path}' is not a folder"` (load_files)

Строки с `{n}`/`{path}`: в dict шаблон с тем же плейсхолдером, подстановка в Rust после `localize`.

`ConfirmModal` / места с `'static str`: сменить пропсы на `Signal<String>` или вызов `localize` внутри `view!` (`move || localize(...)`), чтобы смена языка обновляла открытый диалог.

`MenuCommand.label`: хранить **key**, рендерить `localize(label)`.

## Не делать
- Не переводить содержимое VFS, `user-help.md`, preview файлов пользователя.
- Не переводить ошибки fs/import/export/template/deck_gen.
- Не менять архитектуру баров/workspace (только чтение locale + замена литералов).
- Не делать i18n-фреймворк, fluent, gettext.
- Не добавлять языки кроме EN/RU.
- Не реализовывать themes/feedback/progress-ray в этом заходе (кнопку Locale — да).
- Не дублировать тексты и в conf, и в dict: после задачи единственный источник UI-строк — `dict.json5`.

## Scope кода (анализировать и менять ТОЛЬКО это)
- `Cargo.toml` (только `members`)
- `deck_gen_wasm/Trunk.toml`
- `deck_gen_wasm/index.html` (lang/title можно оставить заглушкой — живые значения пишет Rust)
- `deck_gen_wasm/conf/src/api.rs` (LOCALE_KEY; удаление TOOLTIP_* текстов)
- `deck_gen_wasm/locale/` (новый крейт: lib, keys.rs, dict.json5, Cargo.toml)
- `deck_gen_wasm/ui/Cargo.toml`
- `deck_gen_wasm/ui/src/app.rs`
- `deck_gen_wasm/ui/src/bars/activity.rs`
- `deck_gen_wasm/ui/src/buttons/mod.rs`
- `deck_gen_wasm/ui/src/buttons/clear.rs`
- `deck_gen_wasm/ui/src/buttons/download.rs`
- `deck_gen_wasm/ui/src/buttons/load_game.rs`
- `deck_gen_wasm/ui/src/buttons/new_game.rs`
- `deck_gen_wasm/ui/src/buttons/prepare_html.rs`
- `deck_gen_wasm/ui/src/buttons/prepare_pdf.rs`
- `deck_gen_wasm/ui/src/buttons/split_preview.rs`
- `deck_gen_wasm/ui/src/buttons/new_file.rs`
- `deck_gen_wasm/ui/src/buttons/new_folder.rs`
- `deck_gen_wasm/ui/src/buttons/locale.rs` (новый)
- `deck_gen_wasm/ui/src/buttons/feedback.rs` и `theme.rs` — **только если файлы уже есть**
- `deck_gen_wasm/ui/src/icons/mod.rs`
- `deck_gen_wasm/ui/src/icons/activity.rs`
- `deck_gen_wasm/ui/src/modals/confirm.rs`
- `deck_gen_wasm/ui/src/modals/alert.rs`
- `deck_gen_wasm/ui/src/menus/context.rs`
- `deck_gen_wasm/ui/src/windows/explorer/mod.rs`
- `deck_gen_wasm/ui/src/windows/editor/pane.rs`
- `deck_gen_wasm/ui/src/windows/editor/tab.rs`
- `deck_gen_wasm/workspace/Cargo.toml`
- `deck_gen_wasm/workspace/src/state.rs`
- `deck_gen_wasm/workspace/src/actions.rs`
- `deck_gen_wasm/workspace/src/commands.rs`
- `deck_gen_wasm/icons/buttons/locale/` (новые PNG)
- прочие файлы и папки репозитория ИГНОРИРУЙ (fs, import, export, template, persist, `deck_gen`, style.css, progress).

## Приёмка
- По умолчанию EN, визуально как сейчас (те же английские тексты).
- Кнопка под Clear переключает EN↔RU, иконка меняется; reload сохраняет выбор.
- Все пункты списка ключей на RU после переключения (меню, модалки, тултипы, empty editor, Games, New File/Folder, status при Clear/New Game/Prepare и prompt rename).
- Смена языка обновляет уже открытый chrome без перезагрузки.
- `cargo test -p deck_gen_wasm_locale` — полнота словаря; check ui+workspace+conf зелёные.
- Ошибки нижних крейтов по-прежнему английские — это ок.

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
