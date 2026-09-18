# Рефакторинг №1: «deck_gen — избыточность, модульность, параллелизация prepare_html / prepare_pdf / prepare_png»

## Общие сведения об анализе
- Проанализирован **только** крейт `deck_gen` (все файлы под `deck_gen/src/*.rs`, `deck_gen/src/**/*.rs`, `deck_gen/Cargo.toml`, `deck_gen/README.md`, `deck_gen/build.rs`, `deck_gen/arch.mermaid`).
- Все остальные папки и крейты репозитория (deck_gen_wasm, prepare_pdf_*, progress_viewer и т.д.) полностью проигнорированы.
- Код **не изменялся**. Только чтение + анализ.
- Использованы только локальные инструменты чтения кода (никаких внешних поисков).
- Анализ проведён на соответствие правилам из `wiki/prompts/refactoring-rules.md` + специальным целям из todo.md (избыточность, модульность/ SRP, оптимизация параллелизма).

## 1. Избыточность (redundancy)
Обнаружены повторяющиеся паттерны, которые можно сделать компактнее без потери читаемости и без введения ненужных абстракций.

### Конкретные места
- **Прогресс-бар boilerplate** (самое очевидное дублирование):
  - `lib.rs`: `prepare_html_named`, `prepare_png_named`
  - `pdf_engine/mod.rs`: `prepare_pdf_named`
  - Почти идентичный код:
    ```rust
    progress.set(0.0);
    ... = conf...
    let decks = catalog::find_decks(...)?
    progress.set(10.0 или 15.0);
    let n = decks.len().max(1);
    for (i, deck) in decks.into_iter().enumerate() {
        let base = X.0 + 70.0 * (i as f32) / (n as f32);
        progress.set(base);
        ... работа ...
        progress.set(base + Y);
    }
    progress.set(100.0);
    ```
  - Внутри png ещё дополнительный вложенный цикл с `card_count.max(1)`.
  - Итог: ~30 строк почти копипаста + хрупкие магические числа (10/15/20/70/5/60).

- **Загрузка conf + catalog + запуск** в CLI:
  - `cli.rs`: `html_command`, `pdf_command`, `png_command` + `list_command`.
  - `pdf_command` и `png_command` дублируют:
    - `let loaded = conf()?;`
    - запуск `Chrome::launch(...)`
    - `let p = CliProgress::default();`
    - `pollster::block_on( crate::prepare_*_named(...) )`
    - последующий `for (label, art) in artifacts { print_html_logs(...) ... }`
  - `chrome_locator` + `resolve_maybe_relative` используются только в этих двух местах.

- **Вызов prepare_html внутри prepare_pdf / prepare_png**:
  - `pdf_engine/mod.rs:91` и `lib.rs:135`: `let html = crate::render::prepare_html(...)` вызывается для получения `card_count` и путей, хотя сама функция пишет side-эффекты (cards/html/*.html).
  - prepare_png_named вызывает render prepare_html даже если карточки уже могли быть подготовлены.

- **Мелкие повторы**:
  - Два похожих блока обработки dotted query в `catalog.rs:name_matches_query`.
  - В `cli.rs` дублирование `print_html_logs` вызовов с 4-5 аргументами.
  - `decks.len().max(1)` и `(i as f32) / (n as f32)` встречается 4+ раза.
  - В `render.rs` два Environment (raw + scss) создаются почти одинаково.
  - `conf()` вызывается в CLI даже когда prepare_* внутри сами делают `conf::load`.

- **Cargo.toml**:
  - Комментарии уже присутствуют, но не везде единообразны (одни — над строкой, некоторые inline).
  - Висячих зависимостей не обнаружено (все используются: clap/pollster только под cli, lopdf только в pdf_engine, regex — card+catalog, grass — render и т.д.).
  - Можно сделать комментарии короче и в одном стиле.

### Предлагаемые решения (избыточность)
Решение №1 (рекомендуемое): Выделить небольшой приватный helper внутри модуля (не новую публичную абстракцию):
- `fn with_deck_progress<F>(progress: &impl ProgressHandler, base: f32, total_share: f32, i: usize, n: usize, f: F)` или просто функцию `progress_for_deck`.
- Общий `fn prepare_deck_artifacts(...)` или `process_decks` который принимает замыкание на обработку одной колоды. Оставляет три prepare_* тонкими обёртками.
- Прогресс-логику вынести в один модуль/функцию (или в progress_viewer, но т.к. только deck_gen — внутри).

Решение №2: Принять, что 3-4 места с похожим циклом — это "целенаправленное дублирование" (разные проценты, разный объём работы на итерацию). Оставить как есть, только вынести `let n = ...max(1)` в утилиту `fn deck_count_or_one(len: usize) -> usize`.

Решение №3: Ввести очень маленький trait `DeckProcessor` + blanket impl, но это уже over-engineering (против KISS/YAGNI).

Сравнительная таблица (избыточность):

| Решение | Объём изменений | Риск | Соответствие KISS | Комментарий |
|---------|------------------|------|-------------------|-------------|
| №1 (helper + замыкание) | Средний | Низкий | Отлично | Убирает 60-70% дублирования прогресса и цикла, без новых публичных типов |
| №2 (только утилита len) | Минимальный | Нет | Хорошо | Только косметика |
| №3 (trait) | Большой | Средний | Плохо | Пустая абстракция ради 3 вызовов |

***

Моё заключение: принимаем решение №1, но без изменений в `progress_viewer`.

## 2. Модульность и Single Responsibility Principle
Анализ по принципу "один модуль — одна ответственность".

### Нарушения / проблемы
- **render.rs** (наиболее явное):
  - `prepare_html` + `render_html` делают две разные вещи:
    1. Генерация полных face/back/preview HTML.
    2. Как side-effect — генерация `cards/html/card-N-*.html` для каждой карточки.
  - Комментарий в коде прямо говорит "as side-effect". Это нарушает SRP: потребитель, которому нужны только sheet'ы, получает ненужные файлы на диске. Потребитель PNG зависит от этого side-effect'а.
  - `render_template` и `template_context` — ок, но вся логика записи + создание cards dir живёт внутри.

- **catalog.rs**:
  - Отвечает за discovery колод + загрузку имени.
  - `declared_name` делает regex-парсинг сырой строки только чтобы вытащить `"name"`, потому что на этапе discover плейсхолдеры ещё не раскрыты. Это "знание о формате data.json5" внутри catalog.
  - `name_matches_query` содержит legacy fallback-логику (два похожих split_once).
  - Смешивает "найти файлы" и "загрузить Deck" (хотя `load_deck` делегирует).

- **conf/mod.rs**:
  - Загрузка root + discovery игр + разрешение путей + глобальный `OnceLock<Arc<Conf>>` кэш + fallback + несколько публичных утилит `output_dir_for_name` и т.д.
  - `conf()` и `set_conf` — это глобальное состояние (противоречит правилу про статические переменные).
  - `discover_games` — рекурсивный BFS с pruning — сложная ответственность для conf.

- **cli.rs**:
  - Выполняет роль и "main dispatcher", и "прогресс-реализация для CLI", и "chrome locator builder", и "pretty print".
  - `run()` + 4 command-функции + 3 приватных хелпера. Для маленькой утилиты терпимо, но растёт.

- **pdf_engine/**:
  - `mod.rs` содержит и trait `PdfEngineGenerator`, и `prepare_pdf*`, и re-export impose.
  - impose/* — хорошо разбито (form/import/sheet), но `impose_duplex_bytes` + `page_ids` в mod.rs.
  - `HostPdfEngine` в отдельном модуле под cfg — хорошо.

- **load.rs + model.rs + card.rs**:
  - Хорошее разделение (load = раскрытие плейсхолдеров, model = структура Deck, card = нормализация одной карточки).
  - Но `Deck::from_manager` делает и split, и normalize, и вставку key/view/name — можно было бы чище.

- **error.rs**:
  - Один enum на всё — по KISS нормально. thiserror используется правильно.

- **fs.rs**:
  - Trait + OsFs impl под cfg — отлично. Комментарий про статическую диспетчеризацию правильный.

- **subst/**:
  - Отличное разбиение: cursor / placeholder / sticky / extract. Маленькие файлы, SRP соблюдается.

- **Cargo.toml + build.rs**:
  - build.rs делает post-build copy через spawn powershell/sh — это инфраструктура, а не бизнес-логика deck_gen. Живёт в крейте только потому что "удобно". Нарушает SRP крейта (библиотека + бинарь + build hack).

### Предлагаемые решения (модульность)
Решение №1: Для render — разделить:
- `render_sheets(...) -> HtmlArtifacts` (только face/back/preview)
- `render_per_card_htmls(...)` (отдельная функция, вызывается явно из prepare_png / prepare_pdf при необходимости).
- Или сделать флаг/опцию "emit_card_html_artifacts: bool".

Решение №2: Вынести глобальный кэш conf в отдельный мини-модуль `conf/cache.rs` или полностью убрать кэш для CLI (всегда грузить явно) — см. правило про статические.

Решение №3: catalog::declared_name переименовать/вынести в load как "lightweight name peek", или документировать почему regex неизбежен.

Решение №4 (для cli): Вынести `CliProgress`, `print_html_logs`, `chrome_locator` в отдельные мелкие модули/файлы внутри cli (или в `deck_gen/src/cli/` директорию). Для маленького бинаря — опционально.

Решение №5: build.rs копирование — вынести в xtask или отдельный скрипт/justfile (но это уже инфраструктура за пределами deck_gen).

Сравнительная таблица (ключевые модульные проблемы):

| Проблема | Текущее нарушение | Предлагаемое решение | Сложность | Выигрыш |
|----------|-------------------|----------------------|-----------|---------|
| render side-effect | prepare_html пишет per-card файлы | split render_sheets + render_card_artifacts | Низкая | Чёткий контракт |
| conf global cache | OnceLock | убрать или изолировать в cache.rs | Средняя | Соответствие правилам Rust |
| catalog name peek | regex внутри catalog | lightweight peek в load или задокументировать | Низкая | SRP |
| cli.rs монолит | 4 команды + утилиты | мелкие модули (опционально) | Низкая | Читаемость |
| build.rs | инфраструктура в библиотечном крейте | xtask / внешний скрипт | Средняя | Чистота крейта |

***

Моё заключение:
- решение №1: принимается, но без доп флага (`emit_card_html_artifacts`) пусть явно вызывается отдельная функция, но всегда.
- решение №2: надо полностью убрать кэш для cli.
- решение №3: переименовать, вынести в load.
- решение №4: принимается как есть.
- решение №5: избавиться от `build.rs`, вынести в отдельный скрипт за пределами модуля в `start.bat`.

## 3. Оптимизация: параллелизация prepare_html, prepare_pdf, prepare_png + --threads (CLI only)
Специальная цель todo.md.

### Текущее состояние
- Все три функции (`prepare_*_named`) выполняют `for deck in decks { ... }` строго последовательно.
- Внутри png_named — дополнительный последовательный `for j in 0..card_count { 2 × html_to_png }`.
- pdf_named делает 2 × html_to_pdf + impose на каждой итерации.
- prepare_html полностью синхронная.
- prepare_pdf / prepare_png — async (из-за trait `impl Future`), но в реальности в CLI через `pollster::block_on` и `std::future::ready(...)` (блокирующие вызовы Chrome).
- Прогресс — `&impl ProgressHandler` (CliProgress использует `Cell<i32>`) — не thread-safe.
- Chrome (prepare_pdf_host) создаётся один раз на команду и передаётся по ссылке. Параллельная работа с одним экземпляром Chrome — неизвестна по безопасности (анализ только по deck_gen; внешний крейт не смотрим).

### Предлагаемые решения (параллелизация)
Решение №1 (рекомендуемое для HTML, осторожное для PDF/PNG):
- Добавить под `#[cfg(all(feature = "cli", not(target_arch = "wasm32")))]` зависимость `rayon` (опционально).
- В `Cli` (clap) добавить **глобальный** флаг:
  ```rust
  #[arg(long, default_value_t = 1, global = true)]
  threads: usize,
  ```
  (или только на Html/Pdf/Png subcommands).
- Изменить сигнатуры (или добавить `_with_concurrency` варианты):
  ```rust
  pub fn prepare_html_named<F>(..., concurrency: usize, ...) -> ...
  // аналогично для pdf/png (async)
  ```
- Внутри:
  ```rust
  if concurrency <= 1 {
      for ... { ... }
  } else {
      // rayon
      use rayon::prelude::*;
      let results: Vec<_> = decks.into_par_iter().map(|deck| { per_deck(...) }).collect();
  }
  ```
- Прогресс: заменить Cell → `std::sync::atomic::AtomicI32` (или Arc<Mutex> / crossbeam channel). Или сделать прогресс "грубым": каждая колода даёт 100/N % одним махом.
- Для PDF/PNG: оборачивать engine в `Arc<Mutex<E>>` (или `&E` под rayon scope с осторожностью). Если Chrome не позволяет настоящую параллельность — автоматически падать до 1 или использовать только для подготовки HTML, а engine — под lock.
- В CLI: `let conc = cli.threads; ... prepare_*(..., conc, &p)`

Решение №2 (без внешних крейтов):
- `std::thread::scope` + `join`.
- Плюс: нет новых зависимостей.
- Минус: больше ручного кода, хуже ergonomics с rayon par_iter, нет work-stealing.

Решение №3 (только подготовка HTML + serial engine):
- Параллелить только `prepare_html` (чистый CPU + FS write).
- В prepare_pdf / prepare_png: сначала параллельно подготовить все HTML (или все per-card), потом последовательно прогнать engine + impose (самый тяжёлый и потенциально не-parallel Chrome кусок).
- Это безопаснее и проще.

Решение №4 (будущий tokio):
- Перейти на async runtime. Слишком тяжело для текущего проекта (против KISS).

Сравнительная таблица (параллелизация):

| Критерий                    | №1 (rayon + --threads) | №2 (std threads) | №3 (html parallel + engine serial) | №4 (tokio) |
|-----------------------------|------------------------|------------------|------------------------------------|------------|
| Новые зависимости           | rayon (opt, cli-only) | Нет             | Нет                               | tokio     |
| Сложность реализации        | Средняя                | Средняя         | Низкая                            | Высокая   |
| Безопасность для Chrome     | Требует Mutex / guard  | То же           | Отличная (engine serial)          | ?         |
| Прогресс                    | Нужно фиксить          | То же           | Проще (грубый)                    | Сложно    |
| Производительность (много колод) | Отличная            | Хорошая         | Хорошая для HTML                  | ?         |
| Соответствие текущей async  | Норм (через block_on в threads) | Норм       | Отличное                          | Полная переделка |
| KISS / YAGNI                | Приемлемо (опционально) | Хорошо         | Лучше всего                       | Плохо     |

**Почему другие решения не всегда приемлемы:**
- Полная параллельность PDF/PNG без Mutex'а над Chrome может привести к race / сломанным сессиям CDP — поэтому №3 или guard обязателен.
- --threads=0 или "auto" (num_cpus) можно, но по умолчанию 1 — чтобы не ломать существующий опыт и не создавать проблемы с прогрессом/логами.
- Параллельный прогресс в CLI (println) без синхронизации будет мешаться — нужно или отключать детальный прогресс при threads>1, или агрегировать.

### Что нужно будет сделать при реализации (кратко)
1. Добавить `threads: usize` в clap (global или per subcommand).
2. Сделать `CliProgress` thread-safe.
3. Refactor трёх prepare_*_named чтобы принимать `concurrency: usize`.
4. Внедрить rayon под cfg(not(wasm)) + feature gate если нужно.
5. Для png: можно параллелить и внутренний цикл по карточкам (отдельный rayon scope).
6. Обновить документацию и help.
7. Добавить тесты на threads=1 (дефолт) и threads=N (если есть несколько колод).
8. Не трогать публичный API без необходимости (оставить старые fn как обёртки с concurrency=1).

***

Моё заключение:
- выбираем подход №1 через rayon
- добавляем новый CLI параметр --concurrency:
    - если True - параллелим выполнение при помощи rayon
    - если False - выполняем последовательно
    + в сборке для wasm параллельное выполнение запрещено.

## 4. Дополнительные замечания по правилам рефакторинга
- Идиоматичность Rust в целом хорошая: generics + `impl Future`, `Arc<F>` только когда нужно, OnceLock для regex, trait `FileSystem` с предпочтением статической диспетчеризации.
- Избегать статических: главный нарушитель — `static CONF: OnceLock` в conf. Второстепенный — regex OnceLock (но они read-only и оправданы).
- Комментарии модулей: почти все имеют хорошие //! блоки.
- Публичные API задокументированы.
- README.md deck_gen:
  - В целом актуален.
  - Мелкая неточность: упоминает "per-game conf.json5", хотя уже game.json5.
  - Можно добавить параграф про будущий `--threads`.
- build.rs: работает, но "не чистый" (спавнит внешний процесс на каждую сборку).

## 5. Приоритет предложений (мой субъективный)
1. Устранить дублирование прогресса + цикла по колодам (самый большой выигрыш за маленькие изменения).
2. Разделить side-effect per-card HTML в render (чёткость контракта).
3. Добавить --threads + параллелизацию **только** для prepare_html (или с осторожным guard'ом для pdf/png).
4. Убрать/изолировать глобальный кэш CONF.
5. Мелкие чистки (name_matches_query, cli дубли, комментарии в Cargo).

# Ваше заключение:
По каждой из проблем написал свой заключение.
Можно приступать к рефакторингу.

По завершении необходимо проверить работоспособность следующим образом:
В соседней папке рядом с папкой проекта находится папка Deck Games (`../Deck Games`), где находятся игры на которых можно проверять работоспособность.
Работоспособность проверяется на игре new-game (`C:/MyLife/Deck Games/Games/Templates/new-game`).
Необходимо собрать крейт `deck_gen` через `start.bat` и из корня проекта выполнить следующие команды:
1. `deck_gen.exe list`
2. `deck_gen.exe html --game new-game`
3. `deck_gen.exe pdf --game new-game`
4. `deck_gen.exe pdf --game "new-game"`
5. `deck_gen.exe pdf --game new-game --deck deck-1st`
6. `deck_gen.exe pdf --game "new-game" --deck deck-1st`
7. `deck_gen.exe pdf --game new-game --deck "колода №2"`
8. `deck_gen.exe pdf --game new-game --deck "дополнительная колода"`
9. `deck_gen.exe pdf --game "new-game" --deck "дополнительная колода"`
10. `deck_gen.exe png --game new-game`
11. `deck_gen.exe png --game "new-game"`
12. `deck_gen.exe png --game new-game --deck deck-1st`
13. `deck_gen.exe png --game "new-game" --deck deck-1st`
14. `deck_gen.exe png --game new-game --deck "колода №2"`
15. `deck_gen.exe png --game new-game --deck "дополнительная колода"`
16. `deck_gen.exe png --game "new-game" --deck "дополнительная колода"`

Ошибок быть не должно!
Все команды должны отрабатывать корректно!
