# Результат: корректировка задач stage-8 (Дополнительные Задачи №8, phase-III)

## Задача
Проанализировать код крейтов репозитория (deck_gen, deck_gen_wasm/*, prepare_pdf_host, prepare_pdf_web, и текущий progress) и скорректировать формулировки задач 8-го этапа 3-й фазы в `wiki/plan/phase-III/stage-8/*.md` (заполнить специальные блоки <...> точным описанием целей и минимального scope кода). Не менять текст вне блоков, кроме орфографических/пунктуационных правок. См. `wiki/todo.md` и секцию «Дополнительные Задачи №8» в project-development-plan.md.

## было → стало (почему)

**было:**
- В stage-8 файлах блоки «Что необходимо сделать» и «Scope кода» содержали только шаблонные плейсхолдеры `<здесь необходимо описать...>`.
- Описания задач в заголовках файлов были частично неточны (part-1 в обоих, дублирующиеся заголовки в плане).
- Не было точного, основанного на анализе кода, перечня целей и затронутых файлов для введения `progress_viewer` + `ProgressHandler`, проброса прогресса в prepare_* API deck_gen и в prepare_pdf_* крейты, а также для --threads + динамической таблицы в CLI.
- Анализ кода (структура prepare_html/pdf/png, использование progress в actions.rs, зависимости в Cargo.toml sub-крейтов, chrome/web API, cli.rs, render, отсутствие потоков/процессов сейчас) не был отражён в планах этапа.

**стало:**
- Выполнен полный анализ структуры workspace crates, текущей реализации progress (async + paint, только wasm), вызовов prepare в lib.rs/cli.rs/actions.rs, engine traits и методов host/web.
- Заполнены точные разделы в обоих файлах stage-8:
  - progress_obserbability_part_1.md: цели по извлечению в progress_viewer + trait ProgressHandler, проброс последнего параметра в deck_gen prepare_* + host/web методы, использование из CLI и из wasm api-вызовов.
  - progress_obserbability_part_2.md: цели по --threads (1 по умолчанию), различие thread vs separate processes (для chrome), динамическая таблица прогресса с 4 колонками, ширины из conf.json5, интеграция с ProgressHandler.
- Исправлены очевидные опечатки/ошибки (вне блоков): "часть частью"→"частью", "расшить"→"расширить", "по-умолчанию"→"по умолчанию", заголовок part_2 файла, текст № в project-development-plan.md.
- Scope в блоках перечисляет минимальный набор файлов + что не трогать (соблюдение "минимальные изменения", "запрещено менять архитектуру").
- В result.md записан отчёт (файл очищен перед записью).

(почему: задача из todo.md явно требует анализа кода крейтов + корректировки именно через заполнение специальных блоков <...>; формулировки теперь конкретны, отражают реальную структуру кода (prepare_*_named, Chrome методы, VfsFs+OsFs, spawn/process, conf schema), позволяют в будущем выполнять этап-1/этап-2 по инструкциям; правки минимальны и только по назначению.)

## Проверка
- Прочитаны и проанализированы: Cargo.toml (workspace + члены), deck_gen/{cli.rs,lib.rs,pdf_engine/mod.rs,render.rs,conf/*}, prepare_pdf_host/{lib.rs,chrome/mod.rs}, prepare_pdf_web/src/lib.rs, deck_gen_wasm/{workspace/src/{actions.rs,state.rs,lib.rs}, progress/*, export/*, import/*, template/*, */Cargo.toml}, conf.json5, schema.
- Проверены текущие использования progress macros, prepare вызовы, отсутствие --threads и ProgressHandler.
- Отредактированы только stage-8 md (в блоках + мелкие правки орфографии) + точечная правка плана.
- Соответствует требованиям: "текст за пределами данных блоков не меняй, кроме правки пунктуационных и орфографических ошибок".

## Затронутые файлы (в рамках данной корректировки)
- wiki/plan/phase-III/stage-8/progress_obserbability_part_1.md (заполнены блоки + 1 орфо-фикс)
- wiki/plan/phase-III/stage-8/progress_obserbability_part_2.md (заполнены блоки + заголовок + 2 орфо-фикса)
- wiki/plan/project-development-plan.md (фикс дублирующегося №1 → №2)
- wiki/result.md (очистка + новый отчёт)

Действий от пользователя не требуется. Готово к реализации этапа (сначала part-1, затем part-2, с рефакторингом по правилам).
