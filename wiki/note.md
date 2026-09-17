# Note — png per-card (stage-6) — 2026-09-17

Реализация задачи из wiki/plan/phase-III/stage-6/png_for_card.md завершена:
- Минимальные изменения: prepare_png_* + CardPngGenerator в deck_gen/lib (без трогания pdf_engine); png_command в cli; растеризация в prepare_pdf_host (CDP element screenshot) и prepare_pdf_web (canvas png).
- HTML per-card используется как вход (prepare_html вызывается внутри png).
- Проверено: cargo test, все png-вариации команд на new-game (вкл. quoted и кириллицу в именах колод), структура cards/png/card-N-*.png создана.
- Подготовка к wasm: WebPngEngine доступен.
- Выполнен лёгкий рефакторинг (KISS, docs, Cargo/Readme комменты).
- wiki/result.md обновлён (очищен перед записью).

Действий от пользователя не требуется. Готово.
