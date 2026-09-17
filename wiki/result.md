было: фиксированный Explorer в левой панели + разрозненные кнопки New/Load/Feedback/Theme/Locale на activity bar; нет Games панели и меню Settings.

стало (этап-1 прямолинейно + этап-2 по правилам рефакторинга):
- Введён SidebarMode {Hidden, Explorer, Games} + чистая toggle() (с unit-тестами).
- Activity bar перестроен по плану: Explorer, Games (переключают панель + active), PrepareHtml/Pdf, Download, Split, spacer, Clear, Settings (открывает меню).
- Удалены с бара (не из кода логики): NewGame/Load/Feedback/Theme/Locale.
- Левая панель — универсальный контейнер: при Hidden width=0 + пусто; Explorer — прежний; Games — новый.
- Games: две сворачиваемые секции Local/Global (дефолт развернуты), resizable по высоте перетаскиванием (как split editor), full-width Load Local (через общий confirm), Global — list_game_folders() + строки с Settings→Load (add_game_from_github).
- Обобщён github/template/workspace: list_game_folders + install_game(source) + add_game_from_github; legacy new-game делегирует.
- Settings меню: всплывающее (getBoundingClientRect + backdrop + context-menu css), пункты вызывают старые send/cycle/change+reload.
- Иконки: placeholder копии (split для explorer/games, clear для settings); старые feed/new/load/locale удалены из icons/.
- Локализация: добавлены ключи/строки EN+RU.
- CSS: минимальные правила для .games-section, .game-row, .games-full-btn, hresizer, chevron.
- Состояние стартует Hidden (ширина 0). Prepare/Split/Clear/Download не трогают sidebar. Split active по-прежнему работает.
- Крейты прошли cargo check + unit-тесты (sidebar toggle + github filter + template unique/retarget + ui другие = все ок). trunk build ✅ success.
- Рефакторинг (этап-2): добавлены/улучшены module docs, pub fn docs, #[allow(dead_code)] на legacy, почищены реэкспорты, комментарии в Cargo.toml; без изменения поведения/алгоритмов (KISS, один модуль — ответственность, идиоматичный Rust где просто).

(почему: реализовано строго по vscode_like_ui.md "Цели (делать только это)" + "Scope кода". Левая панель теперь контейнер, activity — переключатели вида + прежние действия + Settings меню. GitHub обобщён без дублирования. Не persist, не новая архитектура, не тронуты prepare/split/clear/zip/editor.)

Браузерная проверка: trunk build прошёл (wasm bundle ок). Полноценное E2E (клик по Explorer/Games, переключение Hidden/active, collapse секций, drag высоты, Load Local, Global fetch+load конкретной игры, меню Settings, drag ширины sidebar) — не автоматизировано в этом окружении (нет browser инструментов); ближайший заменитель — сборка + тесты + ручная проверка по http://127.0.0.1:8080 после serve.bat. Регрессий в остальном не обнаружено (scoped).

wiki/note.md обновлён (иконки).