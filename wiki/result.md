# Отчёт: unique_icon (js / j2)

Задача из wiki/plan/phase-III/unique_icon.md выполнена.

## Принципы
- Только два файла редактировались и анализировались: deck_gen_wasm/fs/src/file_kind.rs и deck_gen_wasm/ui/src/icons/files.rs.
- Минимальные изменения. Архитектура не тронута.
- FileKind / kind_of / syntax_name / импорт / preview — без изменений.
- cargo test как единственный способ верификации (по инструкции).

## было → стало (почему)

### file_kind.rs — ExplorerIcon
было: Md | Json | Json5 | Html | Scss | Pdf | Image
стало: + Js, J2 (после Scss)
почему: п.1 задачи. Только для Explorer, с короткими доками в стиле соседей.

### file_kind.rs — explorer_icon()
было: match по "md","json",...,"scss","pdf", image → _ => None
стало: после "scss" вставлены "js" => Js, "j2" => J2
почему: п.2. Только эти расширения (как md/html). lower_ext даёт регистронезависимость ("JS" → Js).

### file_kind.rs — тест explorer_icon_keeps_htm_markdown_empty
было: ... pdf, notes.txt → None
стало: + assert "a.js"==Js, "a.j2"==J2, "SCRIPT.JS"==Js
почему: требование приёмки + покрытие регистра.

### ui/files.rs — FileTypeIcon
было: match ... Scss => ..., Pdf => ..., None => slot
стало: + Js => <JsFileIcon/>, J2 => <J2FileIcon/>
почему: рендер реальной иконки вместо пустого слота.

### ui/files.rs — новые иконки
было: (нет)
стало:
```rust
fn JsFileIcon() { <svg class="file-icon" ... fill="#f1e05a">...<text>JS</text> }
fn J2FileIcon() { <svg ... fill="#10b981">...<text>J2</text> }
```
почему: 16×16, file-icon. Цвета и метки отличаются от JSON ({}), Html, Scss, Pdf, Image. Стиль как у Pdf/Json5 (текст на документе). Копирование формы без введения абстракций.

### ui/files.rs — тест
было: только md/pdf/image кейсы
стало: + "script.js"==Js, "face-layout.j2"==J2
почему: проверка маппинга в UI-слое.

## Рефакторинг (этап-2)
После этапа-1: выровнены доки (`j2 only`). KISS/YAGNI соблюдены — никаких новых утилит/трейтов/общих компонентов (это было бы изменением архитектуры). Стиль базы сохранён (отдельные view! на каждую иконку).

## Верификация
```
cargo test -p deck_gen_wasm_fs --lib   # 21 tests ok
cargo test -p deck_gen_wasm_ui --lib   # 9 tests ok
```
(прогон после этапа-1 и после рефакторинга)

## Приёмка
- explorer_icon("a.js") == Some(Js), то же для j2 и верхнего регистра — покрыто.
- Неизвестные — пустой слот.
- Другие типы без регрессий (логика не менялась).

## Не сделано (по "Не делать")
- FileKind, conf, превью, меню, папки, гибкие иконки.
- Только inline SVG.

Иконки для script.js и face-layout.j2 теперь должны отображаться в Explorer.

(Отчёт ~68 строк)

---

# Отчёт: корректировка задач stage-2 (фаза-3, Доп.№2)

Дата: 2026-09-15. Задача из wiki/todo.md.

## Выполнено
- Проанализирован код `deck_gen_wasm` (основные модули: ui (app, bars, windows/editor/*, explorer, menus, tooltips), workspace (state, split, api), conf/api, корневой style.css).
- Каждая из 5 задач получила:
  - точное описание текущего поведения по коду;
  - чёткое «Что сделать» + «Не делать»;
  - критерии приёмки;
  - **ограниченный список файлов** для анализа (см. раздел "Какой код использовать для анализа" в каждом .md).
- Из-за пересечения close_all + rearrange по EditorTab / tab vecs / menus — создана `tabs_management.md` (единый scope + общий список файлов).
- План и todo обновлены, опечатки в именах исправлены, ссылки починены.
- Никакой код приложения / wasm не изменён — только документация планирования.

## Минимальные скоупы (итог)
- button_annotations: только conf + 7 файлов buttons/*.rs
- change_width_of_fs_tree: только app.rs + style.css
- flex_adjust_width_of_workspace: conf + editor/mod.rs + style.css
- tabs (объединено): workspace/{state,split,api} + ui/windows/editor/{tab,pane,mod} + menus/* + style.css

## Следующие шаги
Реализацию по обновлённым задачам выполнять строго по указанным спискам файлов (для экономии контекста и изоляции изменений).

(Отчёт по мета-задаче)
