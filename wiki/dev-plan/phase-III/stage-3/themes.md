# Задача: «themes»

Добавить циклическое переключение цветовой темы: тёмная → светлая → системная → тёмная. Кнопка на activity bar под `Feedback`.

## Текущее состояние (из анализа `deck_gen_wasm`)
- Весь chrome — тёмная VS Code-подобная палитра в `style.css` `:root` (`--activity`, `--sidebar`, `--editor`, `--fg`, …).
- Часть цветов **захардкожена** мимо переменных (tooltip `#1e1e1e`/`#454545`, модалки, табы, `.code-highlight .*`).
- Activity-иконки — PNG, нарисованные под тёмный `--activity: #333333`. Смены темы нет, `localStorage` темы нет, `prefers-color-scheme` не читается.
- Кнопок под `Feedback` нет (под ним конец `<nav>`). `Feedback` стоит после `Clear`.
- Паттерн 2-состояний: `SplitPreviewButton` + класс `.activity-btn.active` + 4-й PNG `state-active`. Для **трёх** режимов этого мало — нужен класс/data на кнопке.

## Что сделать

### 1. Модель темы (без нового workspace-крейта)
В `ui` простой модуль `ui/src/theme.rs` (не новый cargo-пакет):
- `enum ColorTheme { Dark, Light, System }` + цикл `next()`.
- Старт: **System** (текущий вид сайта не должен измениться до первого клика).
- Persist: `localStorage` ключ `conf::session::THEME_KEY = "deck_gen_wasm.theme"` (`"dark"|"light"|"system"`). Не класть в `Session` / IndexedDB.
- Применить к документу: `document.documentElement.dataset.theme = "dark"|"light"` (для System — резолв через `matchMedia("(prefers-color-scheme: dark)")` + слушатель `change`, чтобы смена ОС обновляла палитру, а кнопка оставалась в режиме System).
- Вызвать apply один раз при монтировании `App`/`LoadedApp`.

### 2. CSS
- `:root, :root[data-theme="dark"]` — нынешние значения переменных (вид 1-в-1 как сейчас).
- `:root[data-theme="light"]` — светлая палитра explorer/editor/tabs/modals/tooltips/activity-bar (светлый фон, тёмный текст, тот же `--accent: #007acc`).
- **`activity-bar` тоже должен применять светлую тему** (существующие PNG кнопок останутся читаемыми, но без видимо окантовки).
- Вынести захардкоженные chrome-цвета (tooltip, modal, tab, status, tree hover/selected, resizer) на переменные. Синтаксис `.code-highlight` — тоже через переменные с dark-дефолтами и light-переопределением, **не** меняя разметку хайлайтера.
- Не переписывать layout grid.

### 3. Кнопка
- `ui/src/buttons/theme.rs`: цикл по клику, тултип `conf::ui::TOOLTIP_THEME` = `"Color theme (dark / light / system)."`, `aria-label` по текущему режиму.
- Иконка: три набора кадров в `icons/buttons/theme/` — `dark-{off,on,click}.drawio.png`, `light-…`, `system-…`. Пока нет оригинала — **скопировать** PNG `split_preview` (или clear) во все имена. В `wiki/note.md` попросить заменить рисунки.
- CSS: показывать нужную тройку по классу на кнопке (`.theme-dark` / `.theme-light` / `.theme-system`), hover/click как у остальных `.activity-btn`.
- Положение в `ActivityBar`: **после `Feedback `**, в самом низу.

## Не делать
- Не менять PNG существующих кнопок и не вводить светлые варианты activity-иконок.
- Не трогать VFS, persist Session, workspace signals, progress, feedback, locale-крейт.
- Не локализовать строки (задача `localization.md`).
- Не добавлять четвёртый режим, не persist через cookie.
- Не перекрашивать preview iframe (контент файлов пользователя).

## Scope кода (анализировать и менять ТОЛЬКО это)
- `deck_gen_wasm/conf/src/api.rs` (THEME_KEY + TOOLTIP_THEME)
- `deck_gen_wasm/ui/src/lib.rs` (mod theme)
- `deck_gen_wasm/ui/src/theme.rs` (новый)
- `deck_gen_wasm/ui/src/app.rs` (вызов apply при загрузке)
- `deck_gen_wasm/ui/src/bars/activity.rs` (одна кнопка после Feedback)
- `deck_gen_wasm/ui/src/buttons/mod.rs`
- `deck_gen_wasm/ui/src/buttons/theme.rs` (новый)
- `deck_gen_wasm/ui/src/icons/mod.rs`
- `deck_gen_wasm/ui/src/icons/activity.rs`
- `deck_gen_wasm/style.css`
- `deck_gen_wasm/icons/buttons/theme/` (новые PNG)
- прочие файлы и папки репозитория ИГНОРИРУЙ.

## Приёмка
- Первый заход на сайт — системная тема.
- Клики по нижней кнопке: Dark → Light → System → Dark; иконка меняется.
- System: следует ОС; смена темы ОС без перезагрузки обновляет палитру, если выбран System.
- Reload сохраняет выбранный режим.
- Split/explorer resize/табы без регрессий.
- `cargo check -p deck_gen_wasm_ui -p deck_gen_wasm_conf` успешен. Юнит-тест на `ColorTheme::next()` цикл из 3 значений.

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
