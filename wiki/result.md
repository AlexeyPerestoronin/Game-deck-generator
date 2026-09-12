# Доработка №1: PDF preview и смена preview-вкладок (`deck_gen_wasm`)

## Смена preview-вкладок

- было: тело панели рисовалось `{move || pane_body(active)}` → стало: одноэлементный `For` с ключом `kind:path` (почему: в Leptos 0.8 два `PreviewPane` подряд склеиваются как один компонент, `path: String` не реактивен, iframe/blob остаются от последней открытой вкладки).
- было: клик по другой Preview-вкладке менял только active в ленте → стало: ключ меняется, старый preview размонтируется, новый монтируется с нужным файлом.
- было: ключ считался inline в двух местах по-разному → стало: `tab_key` / `body_key` (empty / edit:path / preview:path), тест что ключи расходятся по пути и kind.

## PDF: чёрный лист

- было: `<iframe class="preview-frame">` — flex-ребёнок `.editor` с `flex:1; min-height:0` → стало: обёртка `.preview-host` (flex:1, position:relative, белый фон) и iframe `position:absolute; inset:0; width/height:100%` (почему: Chrome PDF viewer при нулевой стартовой высоте flex-iframe часто рисует чёрную страницу и не перерисовывается).
- было: `.editor { height:100%; min-height:0 }` после split отдавал iframe схлопнутую высоту на первом кадре → стало: `.editor-body` забирает оставшуюся высоту колонки, хост iframe имеет конечный прямоугольник до загрузки PDF.
- было: iframe вешался сразу с `src=""` потом blob URL → стало: `<Show when=src не пуст>` — iframe появляется уже с blob URL внутри готового хоста (почему: пустой src + смена на blob на том же узле у встроенного PDF viewer даёт чёрный лист).
- было: тёмная схема страницы просвечивала в PDF viewer → стало: `color-scheme: light` на `.preview-frame`.
- HTML preview использует тот же `.preview-host` (тот же iframe-класс, без отдельной ветки).

## Что не трогали

- было/стало: split, маршрутизация вкладок, auto-open preview — без изменений (доработка только показа тела панели).
- было: Markdown/картинки уже `flex:1` внутри колонки → стало: они просто живут в `.editor-body`, логика та же.

## Рефакторинг (этап-2)

- было: dynamice view без объяснения → стало: шапка `pane.rs` зачем keyed `For` (не абстракция «на будущее», а фиксация примирения view).
- было: дублированный format ключа вкладки → стало: одна `tab_key` для ленты и тела.
- KISS: не делали Signal-path через все preview-типы — remount по ключу проще и нужен iframe’ам всё равно.

## Тесты

- было: 54 теста, ключ тела не проверялся → стало: 55 ok (`cargo test -p deck_gen_wasm --offline`), плюс `body_key_changes_with_path_and_kind`.
