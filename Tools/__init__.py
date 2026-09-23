import os
import shutil
import invoke
import pathlib


@invoke.task()
def remove_python_cache(ctx, safe_mode: bool = False):
    """Remove python cache files (__pycache__) and byte-code (*.pyc)"""
    cwd = pathlib.Path(os.getcwd())
    target_dir = cwd / "Tools"

    for p in list(target_dir.rglob("*")):
        if p.is_dir() and p.name == "__pycache__":
            shutil.rmtree(p)


collection = invoke.Collection("tools")
collection.add_task(remove_python_cache)

from . import harness, crate

collection.add_collection(harness.collection)
collection.add_collection(crate.collection)