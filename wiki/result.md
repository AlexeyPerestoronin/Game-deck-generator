# Результат: remove_save_button (фаза-III)

Задача выполнена минимальными изменениями строго по указанным файлам анализа.
Изменения только в deck_gen_wasm_ui (как предписано). Никакие другие файлы/папки для анализа/правок не использовались.
Автосохранение, persist, Download/Clear и т.д. — не трогались.

## Формат: было → стало (почему)

### 1. deck_gen_wasm/ui/src/bars/activity.rs

было (комментарий + импорт + использование):
```rust
//! Left icon strip: save, ZIP, new/load game, prepare HTML/PDF, split preview, clear.
...
use crate::buttons::{
    ClearButton, DownloadButton, LoadGameButton, NewGameButton, PrepareHtmlButton,
    PreparePdfButton, SaveButton, SplitPreviewButton,
};
...
        <nav class="activity-bar" aria-label="Actions">
            <SaveButton workspace=workspace />
            <DownloadButton workspace=workspace />
```

стало:
```rust
//! Left icon strip: ZIP, new/load game, prepare HTML/PDF, split preview, clear.
...
use crate::buttons::{
    ClearButton, DownloadButton, LoadGameButton, NewGameButton, PrepareHtmlButton,
    PreparePdfButton, SplitPreviewButton,
};
...
        <nav class="activity-bar" aria-label="Actions">
            <DownloadButton workspace=workspace />
```

(почему: п.1 и п.4 задачи — убрать <SaveButton> из ActivityBar; убрать из use; обновить шапку-комментарий, чтобы не упоминал save. Порядок кнопок теперь точно соответствует приёмке: Download первым.)

### 2. deck_gen_wasm/ui/src/buttons/mod.rs

было:
```rust
mod prepare_pdf;
mod save;
mod split_preview;
...
pub use prepare_pdf::PreparePdfButton;
pub use save::SaveButton;
pub use split_preview::SplitPreviewButton;
```

стало:
```rust
mod prepare_pdf;
mod split_preview;
...
pub use prepare_pdf::PreparePdfButton;
pub use split_preview::SplitPreviewButton;
```

(почему: п.2 задачи — удалить `mod save;` и `pub use save::SaveButton;`. Модуль save.rs удалён полностью, без остаточных ссылок.)

### 3. deck_gen_wasm/ui/src/icons/mod.rs

было:
```rust
pub use activity::{
    ClearIcon, DownloadIcon, LoadGameIcon, NewGameIcon, PrepareHtmlIcon, PreparePdfIcon, SaveIcon,
    SplitPreviewIcon,
};
```

стало:
```rust
pub use activity::{
    ClearIcon, DownloadIcon, LoadGameIcon, NewGameIcon, PrepareHtmlIcon, PreparePdfIcon,
    SplitPreviewIcon,
};
```

(почему: п.3 задачи — убрать реэкспорт SaveIcon из icons/mod.rs.)

### 4. deck_gen_wasm/ui/src/icons/activity.rs

было (весь компонент в начале файла):
```rust
#[component]
pub fn SaveIcon() -> impl IntoView {
    view! {
        <span class="activity-icon" aria-hidden="true">
            <img class="state-off"   src="icons/buttons/save/off.drawio.png"   width="16" height="16" />
            <img class="state-on"    src="icons/buttons/save/on.drawio.png"    width="16" height="16" />
            <img class="state-click" src="icons/buttons/save/click.drawio.png" width="16" height="16" />
        </span>
    }
}

#[component]
pub fn DownloadIcon() -> impl IntoView {
```

стало:
```rust
#[component]
pub fn DownloadIcon() -> impl IntoView {
```

(почему: п.3 задачи — удалить определение SaveIcon. Изменение минимальное: только вырезан блок, без правки остального кода или стиля.)

### 5. deck_gen_wasm/ui/src/buttons/save.rs

было: файл существовал (полная реализация SaveButton + tooltip + вызов persist + SaveIcon).

стало: файл полностью удалён (команда Remove-Item).

(почему: п.2 задачи — "Удалить модуль `buttons/save.rs`".)

## Проверка приёмки (по пунктам из задачи)

- Activity Bar не содержит кнопку Save: `aria-label="Save"` удалён вместе с компонентом. (подтверждено diff'ом в activity.rs)
- Порядок оставшихся кнопок: Download, New Game, Load Game, Prepare HTML, Prepare PDF, Split preview, spacer, Clear. (Save удалён из начала, остальное не тронуто.)
- Перезагрузка страницы по-прежнему восстанавливает workspace — не изменяли autosave (app.rs и persist игнорировались по инструкции).
- Сборка `deck_gen_wasm_ui` без мёртвых импортов SaveButton/SaveIcon: подтверждено.

## Выполненные команды верификации (как указано в задаче)

```
cargo test -p deck_gen_wasm_ui --lib
```
Вывод: exit 0, 9 тестов пройдено (в т.ч. icons, windows/editor и др.). Нет ошибок компиляции по Save*.

```
cargo check -p deck_gen_wasm
```
Вывод: exit 0, Finished `dev` profile. Нет ошибок/предупреждений о неразрешённых SaveButton/SaveIcon.

Крейты, подвергшиеся изменениям, проходят сборку и unit-тесты.

## Что НЕ делали (строго по "Не делать" и "минимальные изменения")

- Не удаляли/не меняли Workspace::persist, deck_gen_wasm_persist, autosave debounce.
- Не трогали кнопки Download, New Game, Load Game, Prepare*, Split, Clear.
- Не меняли ключи localStorage/IndexedDB.
- Не читали и не редактировали никакие другие .rs файлы вне разрешённого списка (только 5 указанных).
- Не удаляли PNG-ассеты иконок (save/off.drawio.png и т.п.) — это другие файлы, игнорируем по инструкции.
- Не проводили рефакторинг (этап-2) — т.к. refactoring-rules.md не входит в разрешённый список файлов для анализа, а требование "Изменения в коде должны быть минимальными!" имеет приоритет. Код правился прямолинейно.

## Верификация UI в браузере

Правило требует открыть приложение в браузере, кликать, проверять все маршруты/состояния, desktop+mobile, искать регрессии.

Доступных инструментов браузера (открытие, screenshot, взаимодействие) в данном сеансе не предоставлено.
В качестве ближайшего заменителя использованы:
- cargo test + cargo check (прошли);
- статический анализ только разрешённых файлов (гарантирует отсутствие aria-label="Save" и импортов).

Полноценную end-to-end проверку в живом браузере (рендер ActivityBar, порядок кнопок, отсутствие Save, работа остальных кнопок + восстановление после reload) выполнить не удалось. Рекомендуется ручная проверка после сборки wasm.

## Итог

Задача решена. Изменения точечные, архитектура не затронута, требования приёмки выполнены в пределах доступных средств.
Отчёт ~85 строк.

## Пост-действия (см. note.md при необходимости)
```
