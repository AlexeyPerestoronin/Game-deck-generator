import invoke
import shutil
import pathlib

import Tools as tools
import Tools.utils as utils


@invoke.task()
def remove_python_cache(ctx):
    """Remove python cache files (__pycache__) and byte-code (*.pyc)"""
    cwd = pathlib.Path(utils.settings.get_cwd())
    target_dir = cwd / "Tools"

    for p in list(target_dir.rglob("*")):
        if p.is_dir() and p.name == "__pycache__":
            shutil.rmtree(p)


@invoke.task()
def make_task_template(ctx, type: str, name: str):
    """Creates template for new task"""
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
        "planning": "planning.md",
        "refactoring": "refactoring.md",
    }
    if type not in templates:
        raise ValueError(f"Unknown task type: {type}")

    src = cwd / "WiKi" / "templates" / templates[type]
    dst_dir = task_dir / f"{index} {name}"
    dst_dir.mkdir()
    shutil.copy(src, dst_dir / "todo.md")


namespace = invoke.Collection()
namespace.add_task(remove_python_cache)
namespace.add_task(make_task_template)

namespace.add_collection(tools.collection)
