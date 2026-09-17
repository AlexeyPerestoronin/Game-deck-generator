# Note — html per-card implementation (stage-6) — 2026-09-17

Реализация задачи из wiki/plan/phase-III/stage-6/html_for_card.md завершена:
- Изменения только в deck_gen/src/render.rs (добавлен helper render_template + генерация cards/html/card-N-*.html как side-effect).
- Полные листы (face/back/preview) и вся остальная цепочка (lib, cli, pdf_engine) без правок.
- Проверено: cargo test, все указанные команды list/html/pdf на new-game (включая quoted и кириллицу).
- Структура и содержимое per-card HTML соответствует спецификации (1-карточный срез, standalone).
- Выполнен рефакторинг (этап-2): KISS, короткие комментарии, стиль базы, обновление модульного док-комментария. Cargo.toml и README проанализированы — изменений не потребовалось.

Действий от пользователя не требуется. Готово.
