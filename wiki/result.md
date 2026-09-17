было: `.code-input { caret-color: #fff; }` (хардкод белый, невидим в light теме на --editor:#ffffff) → стало: в `:root, :root[data-theme="dark"]` добавлено `--caret: #fff;`, в `:root[data-theme="light"]` — `--caret: #000;`; `.code-input { caret-color: var(--caret); }` и `.editor-area { caret-color: var(--caret); }` (для единообразия)

(почему: caret в HighlightedEditor прозрачный текст + отдельный caret; в light нужен контрастный чёрный. Следовали указаниям cursor_bug.md: только style.css, без theme.rs/JS/тестов CSS. В dark оставили белый.)

Этап-1: прямое простое исправление.
Этап-2: рефакторинг не потребовал изменений (код минимальный, стиль CSS базы соблюдён — объявления var в блоках :root по аналогии с --fg/--editor и др.).
Крейт deck_gen_wasm_ui (владеющий UI) прошёл `cargo check` и `cargo test` (10/10 unit-тестов ок, только pre-existing warning).
Браузерная визуальная проверка: недоступна (нет browser automation инструментов в окружении); использован ближайший заменитель — инспекция CSS + успешная сборка+тесты. Per spec: сборка UI не обязательна для чистого CSS.

Регрессий нет (scoped change).