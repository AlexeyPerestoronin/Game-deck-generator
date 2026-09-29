import invoke
import shutil
import pathlib

import tools
import tools.utils as utils


@invoke.task()
def remove_python_cache(ctx):
    """Remove python cache files (__pycache__) and byte-code (*.pyc)"""
    cwd = pathlib.Path(utils.settings.get_cwd())
    target_dir = cwd / "Tools"

    for p in list(target_dir.rglob("*")):
        if p.is_dir() and p.name == "__pycache__":
            shutil.rmtree(p)


@invoke.task(help={
    "type": "type of template (could be on of: development, refactoring, planning, fixing)",
    "name": "short name for the task",
})
def make_task_template(ctx, type: str = None, name: str = None):
    """Creates template for new task"""
    if type is None: raise Exception(f"Error: The '--type' argument is required to create a template (use --help)")
    if name is None: raise Exception(f"Error: The '--name' argument is required to create a template (use --help)")

    cwd = pathlib.Path(utils.settings.get_cwd())
    task_dir = pathlib.Path(utils.settings.get_task_dir())

    indices = []
    for p in list(task_dir.iterdir()):
        if not p.is_dir():
            continue
        prefix = p.name.split(" ", 1)[0]
        if prefix.isdigit():
            indices.append(int(prefix))
    index = max(indices) + 1 if indices else 1

    templates = {
        "development": "development.md",
        "refactoring": "refactoring.md",
        "planning": "planning.md",
        "fixing": "fixing.md",
    }
    if type not in templates:
        raise ValueError(f"Unknown task type: {type}")

    src = cwd / "WiKi" / "templates" / templates[type]
    dst_dir = task_dir / f"{index} {name}"
    dst_dir.mkdir()
    shutil.copy(src, dst_dir / "todo.md")

    settings_template = cwd / "WiKi" / "templates" / "settings_template.json5"
    shutil.copy(settings_template, dst_dir / "settings.json5")


@invoke.task()
def list_tool_tests(ctx):
    """Get list of all unittest of python-tools for this repository"""
    # TODO: необходимо реализовать
    # 
    # Необходимо найти все unittest в директории ./tools и сформировать список доступных unit-тестов для запуска через run_tool_test
    ...

@invoke.task(help={
    "name": "name of the unittest which should be run",
})
def run_tool_test(ctx, name: str = None):
    """Run unittest of python-tools for this repository"""
    # TODO: необходимо реализовать
    # 
    # Запускает целевой тест через python -m unittest <name>
    # Если name = None - запускает все доступных тесты.
    ...


namespace = invoke.Collection()
namespace.add_task(remove_python_cache)
namespace.add_task(make_task_template)

namespace.add_collection(tools.collection)
