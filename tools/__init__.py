"""Invoke-задачи сборки deck_gen и обслуживания Tools."""

import os
import pathlib
import shutil
import subprocess
import sys

import invoke

_CARGO_MANIFEST = "projects/Cargo.toml"
_TRUNK_CONFIG = "projects/Trunk.toml"
_RELEASE_DIR = pathlib.Path("projects") / "target" / "release"
_WEBSITE_PORT = 8080
_DECK_GEN_STEM = "deck_gen"


def _run(command: list[str], cwd: pathlib.Path) -> None:
    # вывод команды сразу в текущую консоль (без перехвата stdout/stderr)
    subprocess.run(command, cwd=cwd, check=True, stdout=sys.stdout, stderr=sys.stderr)


def _copy_release_binaries(root: pathlib.Path) -> None:
    # копируем собранные бинарники из cargo release в корень репозитория
    release_dir = root / _RELEASE_DIR
    for item in release_dir.iterdir():
        if not item.is_file():
            continue
        if item.stem != _DECK_GEN_STEM:
            continue
        is_windows_exe = item.suffix.lower() == ".exe"
        is_unix_bin = item.suffix == "" and os.access(item, os.X_OK)
        if is_windows_exe or is_unix_bin:
            shutil.copy2(item, root / item.name)


def _pids_listening_on_port(port: int) -> list[str]:
    # pid-ы процессов в состоянии LISTENING на указанном порту
    pids: list[str] = []
    if sys.platform == "win32":
        result = subprocess.run(["netstat", "-ano"], capture_output=True, text=True, check=False)
        for line in result.stdout.splitlines():
            parts = line.split()
            if len(parts) < 5 or parts[3].upper() != "LISTENING":
                continue
            if parts[1].endswith(f":{port}") and parts[-1].isdigit():
                pids.append(parts[-1])
    else:
        result = subprocess.run(
            ["lsof", "-ti", f"tcp:{port}"],
            capture_output=True,
            text=True,
            check=False,
        )
        pids = [p for p in result.stdout.split() if p.isdigit()]
    return list(dict.fromkeys(pids))


def _command_line(pid: str) -> str:
    # командная строка процесса — нужна, чтобы запустить его снова
    if sys.platform == "win32":
        result = subprocess.run(
            [
                "powershell",
                "-NoProfile",
                "-Command",
                f"(Get-CimInstance Win32_Process -Filter 'ProcessId={pid}').CommandLine",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        return (result.stdout or "").strip()
    result = subprocess.run(
        ["ps", "-p", pid, "-o", "args="],
        capture_output=True,
        text=True,
        check=False,
    )
    return (result.stdout or "").strip()


def _kill_pid(pid: str) -> None:
    # принудительно завершаем процесс; отсутствие процесса не считаем ошибкой
    if sys.platform == "win32":
        subprocess.run(["taskkill", "/F", "/PID", pid], check=False)
    else:
        subprocess.run(["kill", pid], check=False)


def _stop_website() -> list[str]:
    # остановить локальный web-сайт, если он ещё запущен; вернуть его командные строки
    commands: list[str] = []
    for pid in _pids_listening_on_port(_WEBSITE_PORT):
        if int(pid) in (os.getpid(), os.getppid()):
            continue
        command = _command_line(pid)
        if command:
            commands.append(command)
        _kill_pid(pid)
    return commands


def _start_website(commands: list[str]) -> None:
    # повторно запустить ранее остановленные процессы web-сайта
    for command in commands:
        subprocess.Popen(command, shell=True)


@invoke.task()
def build_deck_gen(ctx, release: bool = False):
    """Build native deck_gen via cargo; copy release binary to repo root."""
    root = pathlib.Path(os.getcwd())
    command = ["cargo", "build", "--manifest-path", _CARGO_MANIFEST]
    if release:
        command.append("--release")
    _run(command, root)
    if release:
        _copy_release_binaries(root)


@invoke.task()
def build_deck_gen_wasm(ctx):
    """Build deck_gen wasm via trunk; restart local website around the build."""
    root = pathlib.Path(os.getcwd())
    website_commands = _stop_website()
    try:
        _run(["trunk", "--config", _TRUNK_CONFIG, "build", "--release"], root)
    finally:
        _start_website(website_commands)


collection = invoke.Collection("tools")
collection.add_task(build_deck_gen)
collection.add_task(build_deck_gen_wasm)

from . import harness

collection.add_collection(harness.collection)
