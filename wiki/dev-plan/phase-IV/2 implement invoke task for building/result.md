# Результаты: «implement invoke task for building»

Формат записей: `было → стало (почему)`

## Tools/crate/actions.py
- `cargo_build`: тело было `# TODO: need to implement` → запускает `cargo build --manifest-path "<path>"` и добавляет `--release`, если `debug=False` (задача требует сборки крейта через cargo; `--manifest-path` привязывает сборку к указанному `Cargo.toml` без смены архитектуры invoke-задач)
- `trunk_build`: тело было `# TODO: need to implement` → запускает `trunk build` в каталоге крейта (`parent` от пути к `Cargo.toml`) через `ctx.cd`, с `--release` при `debug=False` (trunk читает `Trunk.toml`/`index.html` из каталога крейта; у invoke 3.x нет `cwd=` у `ctx.run`, поэтому используется штатный `ctx.cd`)
- дублирование флага профиля в двух задачах → вынесен `_profile_flag(debug)` (меньше копипасты, поведение debug/release одно и то же у обеих задач)
- модуль без описания и `__all__` → добавлены module docstring и `__all__` (PEP 257 / правила рефакторинга Python)

## Tools/crate/__init__.py
- пустой файл → коллекция `crate` с задачами `cargo_build` и `trunk_build` по образцу `Tools/harness/__init__.py` (иначе задачи из `actions.py` не видны `invoke`; архитектура коллекций не менялась, только заполнен уже существующий пакет)

## tasks.py
- подключалась только `harness.collection` → дополнительно `crate.collection` (как уже сделано для harness; без этого `invoke crate.*` недоступен)

## Проверка
- `invoke --list` показывает `crate.cargo-build` и `crate.trunk-build`
- dry-run: `cargo build --manifest-path "..." [--release]` и `cd <crate> && trunk build [--release]`
- реальная сборка: `invoke crate.cargo-build --path Projects/progress_viewer/Cargo.toml --debug` → `Finished dev profile`
- полный `trunk build` не гонялся (долгая wasm-сборка); команда проверена через `--dry`

## README
- не менялся (пользовательский README про CLI/WEB, а не про dev-задачи invoke)
