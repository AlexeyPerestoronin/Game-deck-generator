# Задача
Удалить кнопку `Save` с левой панели действий (Activity Bar). Отдельного ручного сохранения не нужно.

## Текущее поведение (факт из кода)
- Кнопка: `SaveButton` в `deck_gen_wasm/ui/src/buttons/save.rs`, первая в `ActivityBar` (`deck_gen_wasm/ui/src/bars/activity.rs`).
- По клику вызывает `Workspace::persist()` — пишет текстовый снимок в localStorage и бинарники в IndexedDB.
- Иконка: `SaveIcon` в `deck_gen_wasm/ui/src/icons/activity.rs`, реэкспорт в `icons/mod.rs`.
- Автосохранение уже есть: `LoadedApp` в `deck_gen_wasm/ui/src/app.rs` (debounce `conf::ui::AUTOSAVE_DEBOUNCE_MS`) вызывает `save_session` / `save_encoded` при изменении VFS/selection/expanded. После перезагрузки страница восстанавливает сессию без кнопки Save.

## Что сделать
1. Убрать `<SaveButton …>` из `ActivityBar`.
2. Удалить модуль `buttons/save.rs` и его `mod`/`pub use` в `buttons/mod.rs`.
3. Удалить `SaveIcon` и его реэкспорт из `icons`.
4. Обновить шапку-комментарий `activity.rs` (сейчас перечисляет save).

## Не делать
- Не удалять и не менять `Workspace::persist`, крейт `deck_gen_wasm_persist`, autosave в `app.rs`.
- Не трогать кнопку Download (ZIP) и Clear.
- Не менять ключи localStorage / IndexedDB.

## Приёмка
- В Activity Bar нет кнопки Save (нет `aria-label="Save"`).
- Порядок оставшихся кнопок: Download, New Game, Load Game, Prepare HTML, Prepare PDF, Split preview, spacer, Clear.
- Перезагрузка страницы по-прежнему восстанавливает workspace (autosave).
- Сборка `deck_gen_wasm_ui` без мёртвых импортов `SaveButton`/`SaveIcon`.

***

Save в UI — только ручной дубль autosave. Persist-слой оставляем как есть.

# Особые указания
1. Изменения в коде должны быть минимальными!
2. Запрещено менять существующую архитектуру!
3. Какой код использовать для анализа:
   - крейт `deck_gen_wasm_ui`: только
     - `deck_gen_wasm/ui/src/bars/activity.rs`
     - `deck_gen_wasm/ui/src/buttons/mod.rs`
     - `deck_gen_wasm/ui/src/buttons/save.rs` (удалить)
     - `deck_gen_wasm/ui/src/icons/mod.rs`
     - `deck_gen_wasm/ui/src/icons/activity.rs` (`SaveIcon`);
   - прочие файлы и папки репозитория ИГНОРИРУЙ;
3. Краткий отчёт (50-100 строк) о проделанной работе в формате `было→стало(почему)` напиши в `wiki\result.md`.
4. Если какие-то пост-действия требуются от меня напиши их в `wiki\note.md`.

# Дополнительные указания

## Как убедиться в правильности решения:
1. крейты, код которых подвергался изменениям, должны проходить сборку и проверку локальными unit-тестами;
   - `cargo test -p deck_gen_wasm_ui --lib`
   - `cargo check -p deck_gen_wasm`

## Как правильно писать код:
1. этап-1: решить поставленную задачу и убедиться в её работоспособнои требуемым образом;
   - код необходимо писать простой, понятный и прямолинейный, без сложных абстракций;
   - если логика кода сложная, но позволяет написать простые unit-тесты для проверки, их надо написать;
2. этап-2: когда поставленная задача будет решена, код, созданный и зафиксированный на этапе-1, необходимо отрефакторить согласно правилам записанным в `wiki\prompts\refactoring-rules.md`
   - при организации кода придерживайся стиля той кодовой базы в которую вносишь изменения;

## Бережливый подход
1. Максимально береги баланс токенов:
   - использовать X-Search ЗАПРЕЩЕНО;
   - использовать Web-Search ЗАПРЕЩЕНО:
     - если необходимо, сформулируй в чате запрос на разрешение поиска с описаем какую информацию хочешь найти и для чего она нужна в рамках решаемой задачи;
