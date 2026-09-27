import os
import shutil
import invoke
import pathlib


@invoke.task()
def build_deck_gen(ctx, debug: bool = True):
    # TODO: необходимо реализовать
    # Команда для сборки: cargo build --manifest-path Projects/Cargo.toml (запускать из корня)
    # В случае сборки в release получившийся результат необходимо копировать в корень репозитория
    # Запуск команды необходимо делать через subprocess
    # Вывод команды должен отображаться с консоли
    ...

@invoke.task()
def build_deck_gen_wasm(ctx):
    # TODO: необходимо реализовать
    # Команда для сборки: trunk --config Projects/Trunk.toml build --release (запускать из корня)
    # Перед сборкой необходимо завершить локальный процесс для web-сайта если ещё нет, а после сборки запустить снова
    # Запуски всех команд необходимо делать через subprocess
    # Выводы команд должны отображаться в консоли
    ...


@invoke.task()
def remove_python_cache(ctx, safe_mode: bool = False):
    """Remove python cache files (__pycache__) and byte-code (*.pyc)"""
    cwd = pathlib.Path(os.getcwd())
    target_dir = cwd / "Tools"

    for p in list(target_dir.rglob("*")):
        if p.is_dir() and p.name == "__pycache__":
            shutil.rmtree(p)


collection = invoke.Collection("tools")
collection.add_task(build_deck_gen)
collection.add_task(build_deck_gen_wasm)
collection.add_task(remove_python_cache)

from . import harness

collection.add_collection(harness.collection)