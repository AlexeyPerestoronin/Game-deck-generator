import invoke
import shutil
import pathlib
import subprocess
import sys
import unittest

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


def _collect_test_ids(suite: unittest.TestSuite) -> list[str]:
    # рекурсивно собираем id тестов из TestSuite
    ids: list[str] = []
    for item in suite:
        if isinstance(item, unittest.TestSuite):
            ids.extend(_collect_test_ids(item))
        else:
            ids.append(item.id())
    return ids


def _discover_tool_test_ids() -> list[str]:
    # находим unittest-модули в ./Tools и возвращаем id доступных тестов
    cwd = pathlib.Path(utils.settings.get_cwd())
    tools_dir = cwd / "Tools"
    loader = unittest.TestLoader()
    suite = unittest.TestSuite()

    for py_file in sorted(tools_dir.rglob("*.py")):
        if "__pycache__" in py_file.parts:
            continue
        name = py_file.name
        if not (name.startswith("test") or name.endswith("_test.py")):
            continue
        rel = py_file.relative_to(tools_dir)
        module_name = "tools." + ".".join(rel.with_suffix("").parts)
        suite.addTests(loader.loadTestsFromName(module_name))

    return _collect_test_ids(suite)


@invoke.task()
def list_tool_tests(ctx):
    """Get list of all unittest of python-tools for this repository"""
    for test_id in _discover_tool_test_ids():
        print(test_id)


@invoke.task(help={
    "name": "name of the unittest which should be run (if not defined run all tests)",
})
def run_tool_test(ctx, name: str | None = None):
    """Run unittest of python-tools for this repository"""
    cwd = pathlib.Path(utils.settings.get_cwd())
    if name is None:
        names = _discover_tool_test_ids()
        if not names:
            print("No tool tests found")
            return
    else:
        names = [name]
    command = [sys.executable, "-m", "unittest", *names]
    subprocess.run(command, cwd=cwd, check=True, stdout=sys.stdout, stderr=sys.stderr)


namespace = invoke.Collection()
namespace.add_task(remove_python_cache)
namespace.add_task(make_task_template)
namespace.add_task(list_tool_tests)
namespace.add_task(run_tool_test)

namespace.add_collection(tools.collection)
