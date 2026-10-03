# result-ai-prompt-templates

- `request_text` / `system_prompt` с английскими абзацами в `engine.rs` → четыре bundled файла `create-game-pt-{en,ru}.md` и `edit-game-pt-{en,ru}.md`; `engine.rs` только выбирает шаблон (`AiRequest` × `locale::get_active_locale()`) и делает текстовую замену плейсхолдеров (правила агента правятся как документы, не как Rust)
- один user-абзац задачи + отдельный system с правилами → один заполненный шаблон и в system, и в первом user/логе (в `ai-models/log` виден выбранный язык, не смесь, `{game_help}` вложен)
- `{game_help}` читается по уже существующему `conf::game_help::PATH` (`game-help.md`; после задачи локализации help путь сменится сам) → имя help-файла в engine не дублируется
- `ai` без локали → зависимость `deck_gen_wasm_locale` (без прямого leptos), чтобы взять текущий `en`/`ru`
- native-тесты только `from_conf` → плюс create+en («new card game» + user prompt), EditGame подставляет `{game}`/`{file}`, ru-файлы не равны en; без сети
