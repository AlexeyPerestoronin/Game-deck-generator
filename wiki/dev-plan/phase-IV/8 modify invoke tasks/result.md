# Результат

- `build_deck_gen`: TODO-заглушка (`...`) → `cargo build --manifest-path Projects/Cargo.toml` через `subprocess` с выводом в консоль; при `debug=False` добавляется `--release` и `deck_gen`/`deck_gen.exe` копируется из `Projects/target/release` в корень репозитория (требование задачи: сборка из корня, subprocess, вывод в консоль, копирование release-артефакта).
- `build_deck_gen_wasm`: TODO-заглушка (`...`) → перед сборкой останавливается процесс, слушающий порт 8080 (если ещё запущен), выполняется `trunk --config Projects/Trunk.toml build --release` через `subprocess` с выводом в консоль, затем тот же процесс запускается снова (требование задачи: subprocess, вывод в консоль, остановить web-сайт → собрать → запустить снова).
- Вспомогательные `_run` / копирование бинарника / stop-start web-сайта: отсутствовали → выделены рядом с задачами (DRY для двух сборок, KISS без отдельных модулей).
