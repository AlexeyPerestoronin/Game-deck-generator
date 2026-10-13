# Роль
Ты профессиональный разработчик (`см. WiKi/harness/roles/senior-developer.md`).

# Задача
Доработать задачу `todo-harness-loop-via-rust.md` по следующим пунктам:

1. один tools - один модуль - одна запись:
Каждый tools должен быть реализован в рамках отдельного модуля `harness/crates/tools/src/<tool-name>.rs` в конце которого должны быть созданы минимальные unit-тесты, гарантирующие его работоспособность.

В harness.json5 информация об используемых tools в рамках цикла записывается в списке, каждый элемент которого может обладать дополнительными настройками.
Пример:
- как было:
```json5
{
    "tool-settings": {
        "shell": [
            "r:findstr",
        ],
        "dirs": [
            // minimal scope
            "w:tools/harness/agents/tools",
            // minimal scope
            "w:WiKi/dev-plan/phase-IV/15 implement run invoke tool",
            // rules and roles
            "r:WiKi/harness/rules",
            "r:WiKi/harness/roles",
        ],
        "files": [
        ],
        "available-file-extensions": [
            // empty list meant allow all extensions
            // ".md",
            // ".py",
        ]
    },
}
```
- как должно стать:
```json5
{
    "tools": {
        "fs": { // fs.rs - крейт объединяющий множество инструментов для работы с файловой системой
            "available-dirs": [
                // minimal scope
                "w:tools/harness/agents/tools",
                // minimal scope
                "w:WiKi/dev-plan/phase-IV/15 implement run invoke tool",
                // rules and roles
                "r:WiKi/harness/rules",
                "r:WiKi/harness/roles",
            ],
            "available-file-extensions": [
                // empty list meant allow all extensions
                // ".md",
                // ".py",
            ],
            "extra-scope": [
                // здесь должны перечисляться файлы, которые можно читать/изменять за пределами разрешённых директорий
            ]
        },
        "terminal": { // terminal.rs - крейт объединяющий множество инструментов для запуска команд из командной строки
            "cwd-list": [
                // здесь будет формироваться список директорий из которых можно выполнять терминальные команды
            ],
            "available-commands": [
                // здесь перечисляются команды, которые доступны для прямого запуска через терминал
                "findstr",
                "cargo",
            ]
        }
        // другие инструменты (каждый в своём крейте) расширяющий инструментарий определённой областью возможностей
    },
}
```

2. Отдельный модуль-crate для логирования:
Поскольку логирование очень важный аспект проектирования, необходимо создать для него отдельный крейт `harness/crates/logger/Cargo.toml`, который будет определять базовый интерфейс логирования, который будет передаваться во все реализации tools и loops.
(придумай какой-нибудь адекватный интерфейс, а я его поправлю если будет необходимость)

3. Путь до git, в `harness.json`:
Должен быть создан tool-модуль `harness/crates/tools/src/git.rs`, реализующий весь необходимый функционал для работы с git (как в python-реализации, новое не придумывай), но путь до исполняемого файла git должен быть записан в файле конфигурации.