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

# Рефакторинг №2: «deck_gen_wasm — избыточность, модульность, мёртвый код, корректность использования deck_gen API»

## Общие сведения об анализе
- Проанализирован **только** код под `deck_gen_wasm/` (все `*.rs`, `Cargo.toml` в deck_gen_wasm и его поддиректориях с исходниками) + `deck_gen/src/lib.rs`.
- Все остальные папки и крейты репозитория полностью проигнорированы (никаких чтений prepare_pdf_*, progress_viewer, корневого Cargo.toml, deck_gen/src кроме lib.rs и т.д.).
- Код **не изменялся**. Только чтение + анализ + поиск дублирования/мёртвого/нарушений SRP.
- Использованы только локальные инструменты (read_file, grep с path=deck_gen_wasm, list_dir только по deck_gen_wasm/*). Без X-Search/Web-Search.
- Анализ проведён на соответствие `wiki/prompts/refactoring-rules.md` + специальным целям задачи (избыточность, модульность/SRP, мёртвый код, правильность deck_gen API).

## 1. Некорректное использование API из deck_gen (цель №4)

### Описание проблемы
В `deck_gen/src/lib.rs` публичные функции имеют такую сигнатуру (с concurrency):

```rust
pub fn prepare_html<F>(fs: Arc<F>, concurrency: bool, progress: &(impl ProgressHandler + Sync)) -> Result<usize>
pub async fn prepare_png<F, E>(fs: Arc<F>, engine: &E, concurrency: bool, progress: &(impl ProgressHandler + Sync)) -> Result<usize>
// prepare_pdf / prepare_pdf_named реэкспортированы из pdf_engine и по аналогии ожидают похожий контракт
```

В коде deck_gen_wasm вызовы:

- `deck_gen_wasm/workspace/src/actions.rs:205`: `deck_gen::prepare_html(fs.clone(), &sub);` (2 аргумента)
- `deck_gen_wasm/workspace/src/actions.rs:242`: `deck_gen::prepare_pdf(fs.clone(), &engine, &sub).await;` (3 аргумента)
- `deck_gen_wasm/workspace/src/vfs_fs.rs:132,142`: тесты `deck_gen::prepare_html(fs, &progress_viewer::NoopProgress);` (2 аргумента)

Несоответствие. Код либо не скомпилируется, либо использует устаревшую перегрузку. WASM всегда должен передавать `false` (параллелизм только под `#[cfg(all(feature="cli", not(target_arch="wasm32")))]`).

### Предлагаемые решения
Решение №1 (прямое исправление):
- Заменить все вызовы на `deck_gen::prepare_html(fs.clone(), false, &sub);`
- Для prepare_pdf: добавить `false` в правильную позицию (fs, engine, false, &sub) или как требует актуальная сигнатура prepare_pdf.
- Исправить 2 теста в vfs_fs.rs (и любые другие).
- Добавить комментарий: "concurrency=false for WASM (rayon only in native cli)".

Решение №2: Добавить тонкие обёртки `prepare_html_wasm` / `prepare_pdf_wasm` внутри workspace, которые жёстко передают false и прячут разницу. Но это маскировка, а не исправление использования.

Сравнительная таблица:

| Вариант          | Плюсы                              | Минусы                          | Соответствие цели |
|------------------|------------------------------------|---------------------------------|-------------------|
| №1 (править вызовы) | Честное использование API, минимум кода, сразу видно | Нужно править тесты + вызовы   | Полное           |
| №2 (обёртки)     | Изоляция WASM-специфики            | Лишний слой, скрывает проблему | Частичное        |

***

Моё заключение: принимается решение №1!

## 2. Мёртвый код (цель №3)

### Описание проблемы
Явно помеченный мёртвый код, оставленный "для совместимости":

- `deck_gen_wasm/template/src/template.rs`:
  - `#[allow(dead_code)] async fn load_template_files`
  - `#[allow(dead_code)] fn has_both_components`
  - `#[allow(dead_code)] fn finish_files`
  - Комментарий: "Keep old list for compat (new-game specific filter used by legacy path if any)."
- `deck_gen_wasm/template/src/github.rs`:
  - `#[allow(dead_code)] pub async fn list_template_blob_paths` (теперь просто делегирует на обобщённый `list_game_blob_paths`)

Эти пути больше не вызываются после обобщения `install_game(source_game, ...)` и `list_game_folders`.

### Предлагаемые решения
Единственное разумное решение: удалить 4 функции + allow + комментарий. Никаких "на будущее" по YAGNI/KISS.

***

Моё заключение: удалить весь мёртвый код!

## 3. Избыточность (цель №1)

### Конкретные места
- **Дублирование решения "что persist-ить как binary"**:
  - `persist/session.rs: is_persisted_as_binary`
  - `persist/binaries.rs: is_persisted_binary`
  - Буквально одинаковый `matches!(kind::kind_of(path), Image | Pdf)`

- **Дублирование извлечения Vfs из Arc<VfsFs>**:
  - `workspace/actions.rs: fn take_vfs`
  - `workspace/vfs_fs.rs` (в тестах) — inline match Arc::try_unwrap

- **Повторяющийся boilerplate кнопок** (12+ раз):
  ```rust
  let tip: &'static str = Box::leak(locale::localize(KEY).into_boxed_str());
  <DelayedTooltip text=tip>
    <button class="activity-btn" ... disabled=... on:click=...>
      <XxxIcon/>
  ```
  (различаются только иконка, disabled, warning_title и обработчик).

- **Дублирование drag-логики для resizer** (3 независимых места):
  - `ui/src/app.rs` (ширина sidebar)
  - `ui/src/windows/editor/mod.rs` (split preview panes)
  - `ui/src/windows/games/mod.rs` (vertical split Local/Global)
  - Почти идентичный паттерн: RwSignal is_dragging + start coords + Effect + Closure mousemove/mouseup + forget.

- **Дублирование тестовых данных**:
  - списки расширений и кейсы is_previewable / classify повторяются в `fs/file_kind.rs`, `workspace/split.rs`, `import/policy.rs`, тестах import.

- **Дублирование структуры prepare_* в actions.rs**:
  prepare_html и prepare_pdf имеют почти копипастный `spawn_local + progress_wrapper + flush_draft + Arc::new(VfsFs) + call deck_gen + take_vfs + set status/warning`.

- **Иконки** (`ui/src/icons/activity.rs`): 13 почти идентичных компонентов, отличающихся только путями к PNG.

### Предлагаемые решения (редукция избыточности)
Решение №1 (рекомендуемое для ключевых дубликатов):
- Перенести `is_persisted_as_binary` (или `is_binary_persist`) в `fs/kind.rs` как публичную fn и использовать в persist/*.
- Вынести `take_vfs` логику в метод `VfsFs::try_into_vfs(self: Arc<Self>) -> Vfs` (или free fn в fs).
- Для кнопок: ввести `ActivityButton` компонент (или leptos `#[component] fn ActivityButton(icon: ..., on_click, disabled, tip_key...)` ) — один раз. Box::leak остаётся только внутри.
- Для drag: выделить общий `use_split_resizer` / `Resizer` или хотя бы хелпер-функцию, которая возвращает (signals, handlers).
- Удалить дублирующиеся тесты (оставить в одном месте, параметризовать).

Решение №2 (для иконок): data-driven рендер (массив путей + map), но только если не усложнит (KISS — возможно оставить, т.к. статично).

Решение №3 (минимальное): только удалить мёртвое + починить API, остальное "оставить как есть" (YAGNI на рефакторинг boilerplate).

Сравнительная таблица (основные избыточности):

| Дубликат                    | Объём | Влияние на размер/поддержку | Рекомендация | Сложность |
|-----------------------------|-------|-----------------------------|--------------|-----------|
| binary predicate            | 2 fn  | низкое                      | №1 (в kind)  | низкая    |
| take_vfs                    | 2 места | низкое                   | №1 (метод)   | низкая    |
| кнопки + Box::leak          | 12+   | высокое (каждый новый action дублирует) | №1 (компонент) | средняя |
| drag resizers               | 3     | среднее                     | №1 (хелпер)  | средняя   |
| prepare_html/pdf в actions  | ~2x   | среднее                     | Выделить общий run_prepare | средняя |
| тесты расширений            | много | низкое                      | Унифицировать | низкая  |
| activity иконки             | 13    | низкое                      | №2 или оставить | низкая |

(Если решение только одно — причина: остальные либо вводят ненужные абстракции, либо противоречат KISS.)

***

Моё заключение:
- решение №1: принимается полностью!
- решение №2: отклоняется → продолжаем записывать полные пути!
- решение №3: отклоняется в пользу решения №1.

## 4. Модульность и SRP (цель №2)

### Описание проблем
- **Workspace как god-object**: `state.rs` (signals + select/tabs/draft/rewrite/forget + async guards + progress_handle) + actions.rs + commands.rs + split.rs. Даже после разделения — слишком много ответственности на "Workspace handle".
- **VfsFs в неправильном месте**: файл `workspace/src/vfs_fs.rs` реализует `deck_gen::fs::FileSystem`. Комментарий прямо говорит "This used to live in the fs crate". Нарушает SRP workspace (workspace должен знать только о своём Vfs, а не о том, как deck_gen его видит).
- **file_kind.rs** — широкая ответственность (import policy + preview + highlight + icons + mime). Хотя централизация расширений — плюс, предикаты можно было бы декомпозировать.
- **Дублирование решений** (см. redundancy) — прямое следствие отсутствия единого места (persist decision, resizer).
- Мелкие: `split.rs` реэкспортирует `is_previewable` (обёртка над kind), `workspace/api.rs` снова реэкспортирует — цепочка.

### Предлагаемые решения
Решение №1:
- Переместить `VfsFs` (и vfs_key) в `fs/` (как `fs/vfs_fs.rs` или `fs/adapters/deck_gen.rs`) или в отдельный мини-крейт-адаптер. workspace будет зависеть от него только когда нужно.
- Выделить из Workspace более узкие типы/модули: `TabManager`, `SelectionModel` (если вырастет).
- Добавить в `fs/kind` fn `is_persisted_as_binary` (см. выше).

Решение №2: Оставить всё как есть — текущая декомпозиция (отдельные крейты browser/conf/fs/import/export/persist/template + workspace/ui) уже довольно хорошая для WASM-приложения. Дополнительное дробление нарушит KISS.

Сравнительная таблица:

| Проблема             | Текущее нарушение SRP                  | Решение №1                     | Решение №2 (статус-кво) |
|----------------------|----------------------------------------|--------------------------------|-------------------------|
| VfsFs                | в workspace                            | переместить в fs/              | терпимо (маленький файл) |
| Workspace            | много сигналов + логики в одном        | выделить под-структуры         | ок для Leptos Copy handle |
| kind.rs              | classification + UI decisions          | split kind + ui_icon_policy    | ок (централизация важнее) |
| drag / boilerplate   | дублирование вместо общего helper      | общие утилиты                  | копипаста               |

***

Моё заключение:
1. Переместить `VfsFs` (и vfs_key) в `fs/` (как `fs/vfs_fs.rs` или `fs/adapters/deck_gen.rs`).
2. Добавить в `fs/kind` fn `is_persisted_as_binary` (см. выше).

## 5. Анализ Cargo.toml (по правилам рефакторинга)
- Большинство зависимостей имеют поясняющие комментарии (хорошо в workspace, ui, import, export, persist).
- Обнаружена висячая зависимость:
  - `deck_gen_wasm/locale/Cargo.toml`: `serde = { version = "1", features = ["derive"] }`
  - В коде `locale/src/*.rs` — ни одного использования serde / Serialize / Deserialize. json5 используется напрямую. → можно удалить.
- `prepare_pdf_web` правильно только в workspace (где используется WebPdfEngine).
- `zip`, `syntect`, `pulldown-cmark`, `gloo-*`, `futures`, `js-sys` и т.д. — все используются.

***

Моё заключение: удалить висячую зависимость.

## 6. Другие замечания
- Соответствие KISS/YAGNI/идиоматичности Rust: в целом хорошее. Много мелких модулей-крейтов, trait-based (FileSystem), signals вместо глобалов.
- Комментарии модулей и pub API: в основном присутствуют и информативны.
- README.md + arch.mermaid: актуальны, отражают текущую структуру (VFSFS → deck_gen).
- Нет мутируемых статических (правильно, всё через сигналы/OnceLock).
- WASM-ограничения (spawn_local, отсутствие rayon) учтены.

## 7. Приоритет предложений (мой субъективный)
1. Исправить использование deck_gen API (блокер для компиляции/корректности).
2. Удалить мёртвый код в template/ (просто и чисто).
3. Устранить дублирование binary-предиката + take_vfs (маленький выигрыш, высокая ясность).
4. Ввести ActivityButton (или эквивалент) для кнопок — уменьшит boilerplate при добавлении новых действий.
5. Переместить VfsFs в fs/ (модульность).
6. Почистить дубли drag / тесты — по желанию (меньший приоритет).

# Ваше заключение:

Отписался по каждому из пунктов (см. `ОМоё заключение: ...`).
Можно приступать к рефакторингу.

## Дополнительные задачи для рефакторинга:
1. Сделать внутреннюю реструктуризацию:
    - все отдельные программные модули в папке `deck_gen_wasm/modules` (fs, persis, workspace, ...)
    - все вспомогательные файлы папке `deck_gen_wasm/addons` (icons, images, ...)
2. Из всех модулей удалить файл `api.rs` (должно быть достаточно только `lib.rs`)

По завершении необходимо проверить работоспособность следующим образом:
- `deck_gen_wasm` долен собираться и запускаться
- работоспособность в браузере я проверю самостоятельно.
