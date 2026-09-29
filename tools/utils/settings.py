"""Чтение настроек проекта из ``tasks.json5``."""

import json
import pathlib

__all__ = [
    "get_cwd",
    "get_task_dir",
]


def _load_settings() -> dict:
    # tasks.json5 лежит в корне проекта (родитель каталога Tools)
    path = pathlib.Path(__file__).resolve().parents[2] / "tasks.json5"
    with path.open(encoding="utf-8") as file:
        return json.load(file)


def get_cwd() -> pathlib.Path:
    """Return the ``cwd`` path from ``tasks.json5``.

    Returns:
        pathlib.Path: Project working directory.

    Raises:
        FileNotFoundError: If ``tasks.json5`` is missing.
        KeyError: If the ``cwd`` key is missing.
    """
    return pathlib.Path(_load_settings()["cwd"])


def get_task_dir() -> pathlib.Path:
    """Return the ``dirs.task`` path from ``tasks.json5``.

    Returns:
        pathlib.Path: Directory that stores task folders.

    Raises:
        FileNotFoundError: If ``tasks.json5`` is missing.
        KeyError: If the ``dirs`` or ``task`` key is missing.
    """
    return pathlib.Path(_load_settings()["dirs"]["task"])
