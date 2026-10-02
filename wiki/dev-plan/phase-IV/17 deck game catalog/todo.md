# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача на разработку: Каталог игр в ЛП-Games (VSCode-like)

Доработать левую панель в режиме Games (далее ЛП-Games) так, чтобы она работала как каталог расширений VSCode: карточки игр с иконкой/именем/меню команд, локальный список из VFS, глобальный каталог с GitHub `Deck-Games`, поиск, html-preview афиши, раздельное хранение сессионных preview.

Зачем: сейчас Globals — плоский список имён папок из `Game-deck-generator`, Local — только кнопка загрузки, нет иконок, афиши, поиска и scoped-генерации. Нужен каталог игр.

Ожидаемый результат: пользователь в ЛП-Games видит Local/Globals в стиле VSCode Extensions; клик по карточке открывает html-preview афиши; Globals ищет игры в https://github.com/AlexeyPerestoronin/Deck-Games; preview-файлы каталога живут в `temporary_vfs` и исчезают при перезагрузке страницы; Prepare HTML/PDF умеет выбирать игры чекбоксами, если в VFS больше одной игры.

## Зафиксированные решения заказчика
1. **Prepare PNG не делать** (ни в Settings карточки, ни на activity bar). Команду из concept.md считать снятой.
2. **Prepare HTML/PDF при нескольких играх в persist-VFS:** при наведении на соответствующую кнопку activity bar — меню со списком загруженных игр и checkbox у имени. Генерация только для отмеченных. Если ни одна не отмечена — предупреждение. Если в VFS ровно одна игра — меню не показывать, генерация сразу для этой игры.
3. **Афиша:** html-preview открывается по клику на карточку игры (как details у VSCode Extensions), не при загрузке списка.
4. **Хранение preview каталога:** отдельный `temporary_vfs`. Persist-VFS (игры пользователя) не смешивать с сессионными данными. Перезагрузка страницы сбрасывает `temporary_vfs`.
5. **Local Icon/Name:** метаданные из `<game-root>/rules/preview/info.json5` внутри persist-VFS этой игры. Имя файла иконки — поле `icon` (в примере заказчика `"icon.png"`), не хардкодить имя файла.

## Принятые уточнения (делать так; не расширять)
Заказчик не закрыл поиск и дубль Prepare в карточке. Фиксируем минимум из concept.md:
- Поиск Globals: клиентский фильтр уже загруженного списка по кнопке Search. Совпадение без учёта регистра по `info.name.EN`, `info.name.RU` и `tags`. Пустой запрос = показать все.
- Команды Prepare HTML/PDF в Settings карточки Local **оставить**: готовят только эту одну игру, без меню чекбоксов.
- Триггер генерации с activity bar при 2+ играх: hover открывает меню (меню живёт, пока курсор над кнопкой или меню — иначе чекбоксы бесполезны); клик по кнопке activity bar запускает генерацию для отмеченных; 0 отмеченных → warning.
- Ветку Deck-Games считать `master` (константа в conf). Если GitHub ответит 404 на tree — остановиться и спросить, не переключать ветку молча.
- Теги в карточке не рисуем, только участвуют в поиске.

Если любое из этих уточнений противоречит тому, что увидишь в `note.md` — остановись и спроси.

## Как устроено сейчас (не ломать лишнее)
ЛП-Games уже есть:
- `SidebarMode::Games`, секции Local/Global, collapse, вертикальный сплит 50/50, кнопка Load Local Games → тот же confirm, что activity Load.
- Globals: `deck_gen_wasm_template::list_game_folders()` — top-level `games/<name>` в репо `AlexeyPerestoronin/Game-deck-generator` (ветка `master`). Имя карточки = имя папки. Settings → единственная команда Load Game → `Workspace::add_game_from_github`.
- Local: только кнопка, списка игр нет.
- `prepare_html` / `prepare_pdf` гоняют **весь** persist-VFS.
- Preview вкладки читают **только** persist-VFS (`Workspace.vfs`). Persist: localStorage + IndexedDB. `Workspace.snapshot()` / autosave сериализуют только persist-VFS.
- Репозиторий шаблона `new-game` (`conf::github`, `install_new_game`) **не менять**.

Ключевые файлы текущей реализации:
- `projects/deck_gen_wasm/modules/ui/src/windows/games/mod.rs`
- `projects/deck_gen_wasm/modules/ui/src/bars/activity.rs`
- `projects/deck_gen_wasm/modules/ui/src/buttons/prepare_html.rs`, `prepare_pdf.rs`
- `projects/deck_gen_wasm/modules/template/src/github.rs`, `template.rs`
- `projects/deck_gen_wasm/modules/workspace/src/state.rs`, `actions.rs`
- `projects/deck_gen_wasm/modules/conf/src/lib.rs`
- `projects/deck_gen_wasm/style.css` (блок `.games-*`)
- `projects/deck_gen_wasm/modules/locale/dict.json5`, `keys.rs`

---

## Функциональные требования

### 1. Карточка «Available Game» (и Local, и Globals)
Как VSCode Extensions list item:
- слева **Icon** (квадрат ~42×42, `object-fit: contain`, фон прозрачный);
- рядом **Name** (ellipsis, одна строка);
- справа **Settings** (кнопка, открывает контекстное меню как сейчас `.context-menu`).
- hover строки: `--hover`; выбранная (открыта её афиша): `--selected`.
- Клик по строке (не Settings) → открыть html-preview афиши этой игры (см. §5).
- Клик по Settings: `stop_propagation`, меню команд, афишу не открывать.

### 2. Local
Секция как сейчас (collapse + header «Local» / «Локальные»), в body сверху кнопка `Load Local Games` (поведение не менять).

Под кнопкой — список игр, найденных в **persist-VFS**:
- Игра = папка, в которой есть файл `game.json5` (обход дерева VFS; см. §7).
- Icon/Name из `{game-root}/rules/preview/info.json5` по схеме §6.
- Нет info.json5 / нет нужных полей / нет файлов → дефолты §6, игру **не** пропускать (она уже в VFS).
- Settings Local:
  1. `Prepare HTML` — html только для колод **этой** игры;
  2. `Prepare PDF` — pdf только для колод **этой** игры.
- Список реактивный: load/delete/rename/clear обновляют карточки без перезагрузки страницы.

### 3. Globals
Секция как сейчас (collapse + header). В body **сверху** контейнер Search:
- `text-field` (placeholder локализован);
- кнопка `search`;
- Enter в поле = нажатие Search.
- Фильтр применяется только по кнопке/Enter, не live-on-each-keystroke.
- Пустой запрос после Search → снова все загруженные в каталог игры.

Ниже — карточки игр, найденных в GitHub-репо каталога (см. §8).

Settings Globals: единственная команда `Load Game` — скачать **всю** папку игры из Deck-Games в persist-VFS под `games/<unique>` (уникальность как сейчас через `unique_name`). `games/conf.json5` из каталога не требовать. После успеха игра появляется в Local.

Состояния: loading, ошибка fetch (текст ошибки), пустой список после успешного fetch.

Fetch каталога: один раз при первом показе Games-панели (как сейчас список папок). Перезапрос при повторном заходе в Games не обязателен (сессия страницы). `temporary_vfs` при этом заполняется preview-файлами.

### 4. `temporary_vfs`
- Новый сигнал на `Workspace`: `temp_vfs: RwSignal<Vfs>`.
- **Не** входит в `snapshot()`, autosave, localStorage, IndexedDB.
- Перезагрузка страницы = пустой `temp_vfs` (ничего не восстанавливать).
- `Workspace::clear()` чистит persist-VFS как сейчас; `temp_vfs` тоже очистить (иначе афиши будут ссылаться на пустой persist, а каталог останется — допустимо очистить оба, это проще и безопаснее).
- Explorer **не** показывает `temp_vfs`.
- Сюда складывать все скачанные `rules/preview/**` глобальных игр и подставляемые default-preview при открытии афиши без файла.

### 5. html-preview афиши
По клику на карточку открыть вкладку `TabKind::Preview` (существующий HtmlPreview / iframe srcdoc).

Источник файла:
- **Local:** persist-VFS `{game-root}/rules/preview/{preview}` где `{preview}` из info.json5 (обычно `preview.html`).
- **Global:** `temp_vfs` по тому же относительному пути репозитория, например `Games/<...>/rules/preview/preview.html`.
- Нет записи `preview` или нет файла → показать bundled `default-preview.html` (открыть как preview; можно положить копию в `temp_vfs` на служебный путь и открыть его).

Чтение preview: HtmlPreview и `inline_relative_iframes` должны читать из **того** VFS, где лежит html (persist или temp). Не изобретать inlining картинок/css — оставить текущее поведение srcdoc (inlining только relative iframe). Если афиша без этого не открывается — остановись и спроси.

Не открывать Edit-вкладку для афиши. Если вкладка с этим path+kind уже есть — активировать её.

### 6. Схема `info.json5`
Путь: `{game-root}/rules/preview/info.json5`. JSON5.

```json5
{
    "preview": "preview.html",
    "icon": "icon.png",
    "info": {
        "name": {
            "EN": "Poker",
            "RU": "Покер"
        },
        "tags": [
            "classic"
        ]
    }
}
```

Правила:
- `preview` — имя файла в той же папке `rules/preview/`. Нет поля или нет файла → default-preview.
- `icon` — имя файла в той же папке. Нет поля или нет файла → bundled `default-icon.png`.
- `info.name.<locale>`: locale UI `en` → `EN`, `ru` → `RU`. Нет ключа текущей локали → fallback `EN`. Нет и `EN` → локализованная заглушка `Name is not defined...` / `Имя не определено...`.
- `tags` — массив строк; нет/не массив → пустой список (игра не отбрасывается).
- Битый JSON5: Local — дефолты; Global (каталог) — игру **пропустить** (как «нет info.json5»).

Bundled defaults (создать файлы):
- `projects/deck_gen_wasm/template/default-preview.html` — простая HTML-страница с текстом, что preview отсутствует (EN и RU на одной странице, без JS).
- `projects/deck_gen_wasm/template/default-icon.png` — небольшой PNG-плейсхолдер (например 64×64), бинарный файл в git.
Подключать через `include_str!` / `include_bytes!` из crate `deck_gen_wasm_template`.

### 7. Детект игр в persist-VFS (Local и меню Prepare)
Чистая функция по списку путей VFS (из `visit_entries`):
- игра = директория, содержащая файл `game.json5` (path вида `{root}/game.json5`);
- `{root}` может быть вложенным; каждая такая папка — отдельная игра;
- сортировка по path для стабильности UI.

Имя в меню Prepare: локализованное из info.json5, иначе заглушка, иначе последний сегмент `{root}`.

### 8. Поиск игр в Deck-Games (Globals)
Репозиторий: `AlexeyPerestoronin/Deck-Games`, ветка `conf` константа `master`.
Корень игр: папка `Games` (с большой буквы).

Алгоритм:
1. GET recursive git tree (как сейчас в `github.rs`, другой repo/branch).
2. Найти все blob, чей path начинается с `Games/` и заканчивается на `/game.json5` либо равен `Games/game.json5`. Это эквивалент BFS по дереву; отдельный постраничный Contents API не делать.
3. Для каждой найденной игры `game-root` = path без суффикса `/game.json5`.
4. Скачать **все blob** под `{game-root}/rules/preview/` в `temp_vfs` (сохранять repo-relative path). Текстовые (`html`, `htm`, `json`, `json5`, `md`, `css`, `scss`, `js`) — `fetch_text`; картинки (`png`, `jpg`, `jpeg`, `ico`, `icon`) — новый `fetch_bytes`. Прочие — bytes.
5. Нет папки preview, нет `info.json5`, битый info → игру **не показывать**.
6. Ошибка скачивания preview одной игры → пропустить игру, каталог в целом не валить.
7. Параллелизм: существующий `map_join` + `conf::io::FETCH_PARALLEL`.

Не использовать больше `list_game_folders()` в UI Globals. Саму функцию для `new-game`/тестов можно оставить.

Load Game: скачать все blob под `{game-root}/` из Deck-Games в persist-VFS как `games/{unique(last segment)}/...` (strip prefix `{game-root}/`). Не ретаргетить deck id, если не уверен — ретаргет как в `install_game` только при смене имени папки (`"from.` → `"to.`).

### 9. Prepare HTML/PDF — скоуп по играм
`deck_gen::prepare_html` / `prepare_pdf` фильтра по игре не имеют. Не менять crate `deck_gen`.

Сделать обёртку в workspace:
1. Собрать working `Vfs`: скопировать `games/conf.json5` если есть + **целиком** поддеревья выбранных `{game-root}`.
2. Прогнать prepare на working Vfs (как сейчас, с progress).
3. Записать обратно в persist-VFS все файлы working, лежащие внутри выбранных `{game-root}` (overwrite). Shared `games/conf.json5` обратно не трогать, если не обязан.

Если 0 игр в persist-VFS и пользователь нажал Prepare на activity bar → warning (новая locale-строка).

Activity bar:
- 1 игра: клик = prepare этой игры, меню нет. DelayedTooltip как сейчас.
- 2+ игр: DelayedTooltip **не** показывать; hover → меню чекбоксов справа от кнопки (fixed, как tooltip, чтобы не клипалось overflow activity bar). Клик по кнопке = prepare отмеченных; 0 отмеченных → warning.
- Чекбоксы по умолчанию сняты. Состояние чекбоксов — UI-local, не persist.

Settings карточки Local: сразу prepare этой игры (тот же backend, список из одного root).

Кнопки disabled пока `workspace.loading`.

### 10. UI/CSS
Держать VSCode dark/light tokens (`--sidebar`, `--hover`, `--selected`, `--fg`, `--fg-dim`).
- `.game-row`: горизонтальный flex, padding ~6–8px, gap, min-height под 42px icon.
- `.game-icon` 42×42.
- Search: input на всю ширину + кнопка; sticky вверху body Globals.
- Меню Prepare: чекбокс + имя; не уезжать за viewport (clamp top/left).
- Не ломать explorer/editor стили.

### 11. Локализация
Все новые UI-строки — ключи в `keys.rs` + EN/RU в `dict.json5` + тест `all_keys_have_en_and_ru`.
Минимум ключи:
- заглушка имени;
- placeholder и кнопка Search;
- Prepare HTML / Prepare PDF в меню карточки (можно переиспользовать существующие aria, если текст совпадает; если нет — отдельные keys);
- warning: нет игр; не выбрана ни одна игра;
- пустой каталог / не найдено по поиску;
- status загрузки каталога / preview.
Не хардкодить английские литералы в UI (сейчас в games есть `(no games listed)` — убрать в locale).

Имя игры из info.json5 должно реагировать на смену локали (читается `localize`/`get_active_locale` в reactive view).

### 12. Архитектура (KISS)
Не создавать новый crate. Расширить существующие:

| Место | Что |
|---|---|
| `conf` | `catalog::{REPO, BRANCH, GAMES_DIR, GAME_MARKER, PREVIEW_DIR, INFO_FILE}` + пути default-файлов |
| `browser` | `fetch_bytes(url) -> Result<Vec<u8>, String>` рядом с `fetch_text` |
| `template` | модуль каталога: parse tree → game roots; parse info.json5; fetch preview blobs; install full game from Deck-Games; bundled defaults. Чистые функции покрыть unit-тестами |
| `workspace` | `temp_vfs`; `open_preview_from(vfs_kind, path)`; `prepare_html_for(game_roots)` / `prepare_pdf_for`; детект local games из persist-VFS |
| `ui/windows/games` | карточки, search, click→preview, Local list |
| `ui/buttons/prepare_*` | hover-меню чекбоксов |
| `ui/windows/editor/preview.rs` (+ iframe) | чтение persist или temp |
| `locale` | ключи |
| `style.css` | карточки, search, icon |
| `README.md` / `arch.mermaid` | одна фраза про каталог Deck-Games + temp_vfs |

Preview lookup: не раздувать `OpenTab`, если можно `read_file/read_bytes`: сначала persist, если нет файла — `temp_vfs`. Пути каталога (`Games/...`) с persist не пересекаются (`games/...` в persist). Если пересечение всё же возможно — явный origin на табе; не плодить абстракции заранее.

---

## Scope
Читай и меняй только:
- `projects/deck_gen_wasm/modules/ui/`
- `projects/deck_gen_wasm/modules/workspace/`
- `projects/deck_gen_wasm/modules/template/`
- `projects/deck_gen_wasm/modules/browser/`
- `projects/deck_gen_wasm/modules/conf/`
- `projects/deck_gen_wasm/modules/locale/`
- `projects/deck_gen_wasm/style.css`
- `projects/deck_gen_wasm/template/` (новые default-preview.html, default-icon.png)
- `projects/deck_gen_wasm/README.md`
- `projects/deck_gen_wasm/arch.mermaid`

Остальное игнорируй. **Не** менять `deck_gen`, `prepare_pdf_web`, persist-формат Session (кроме того что temp_vfs туда не попадает).

## Проверка корректности решения
Обязательные unit-тесты (native, без wasm, в том же стиле что `games/mod.rs` и `github.rs`):
- детекте game-root по списку путей (`Games/a/game.json5` → `Games/a`; вложенность; игнор blob без маркера);
- parse info.json5: полное; нет name; нет icon/preview; битый;
- фильтр поиска: name EN/RU, tag, case, пустой query;
- isolate/merge VFS: в working только выбранные игры + conf; обратно пишутся файлы только под выбранными roots;
- `section_flex` существующие тесты не сломать;
- locale: все новые ключи EN+RU.

Ручная проверка (описать в `result.md`, прогнать если среда позволяет `trunk serve`):
1. Games → Local: после Load Local Games карточка с иконкой и именем; Settings → Prepare HTML/PDF.
2. Клик по Local-карточке открывает Preview афиши (или default-preview).
3. Globals грузит Deck-Games, карточки с иконками; Search фильтрует; Load Game кладёт игру в persist и она появляется в Local.
4. Reload страницы: persist игры на месте, каталог/афиши из temp сброшены (каталог перечитывается при открытии Games).
5. Две+ игры в VFS: hover на Prepare HTML — чекбоксы; 0 отмеченных + клик — warning; отметить одну — готовится только она.
6. Одна игра: hover-меню нет, клик готовит её.
7. Explorer не показывает файлы каталога из temp_vfs.
8. PNG-команд нигде нет.

# Дополнительные указания
1. Экономь токены: читай минимум необходимого; правки вноси через patch.
2. Если что-то не получается со второго раза, или ты понимаешь, что контекст задачи сильно разрастается относительно цели задачи в минимальном воплощении, или что-то какая-то информация не дана, но является важной для правильной реализации поставленной задачи → не фантазируй и не додумывай за меня → остановись и задай вопрос → я подскажу и направлю.
3. Результаты — в `result.md` рядом с `todo.md`, формат `было→стало(почему)`.
4. Действия с моей стороны — в `note.md` рядом с `todo.md`.
