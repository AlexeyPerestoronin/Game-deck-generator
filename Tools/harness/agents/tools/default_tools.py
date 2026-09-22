import os
import shutil
import subprocess

from classproperties import classproperty

from . import i_tools

__all__ = ['DefaultTools']


class DefaultTools(i_tools.ITools):
    """Default tool set: shell execution (with confirm), read/write file (sandboxed)."""

    def __init__(self, safe_mode: bool, settings: dict):
        self._safe_mode = safe_mode
        self._command_execution_limit = settings.get("command-execution-limit", 30)
        self._w_shell = [c[2:] for c in settings.get("shell", []) if c.startswith("w:")]
        self._r_shell = [c[2:] for c in settings.get("shell", []) if c.startswith("r:")] + self._w_shell
        self._w_dirs = [os.path.abspath(d[2:]) for d in settings.get("dirs", []) if d.startswith("w:")]
        self._r_dirs = [os.path.abspath(d[2:]) for d in settings.get("dirs", []) if d.startswith("r:")] + self._w_dirs

        self._tools = [
            i_tools.Tool(n, d, p) for n, d, p in [
                ("read_file", "Прочитать содержимое файла.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                ("write_file", "Записать файл.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        },
                        "content": {
                            "type": "string"
                        }
                    },
                    "required": ["path", "content"]
                }),
                ("create_file", "Создать файл.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                ("remove_file", "Удалить файл.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                ("move_file", "Переместить файл.", {
                    "type": "object",
                    "properties": {
                        "src": {
                            "type": "string"
                        },
                        "dst": {
                            "type": "string"
                        }
                    },
                    "required": ["src", "dst"]
                }),
                ("create_folder", "Создать папку.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                ("remove_folder", "Удалить папку.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                ("move_folder", "Переместить папку.", {
                    "type": "object",
                    "properties": {
                        "src": {
                            "type": "string"
                        },
                        "dst": {
                            "type": "string"
                        }
                    },
                    "required": ["src", "dst"]
                }),
                ("list_available_shell_commands", "Список доступных shell команд.", {}),
                ("run_shell", "Запуск команды.", {
                    "type": "object",
                    "properties": {
                        "cwd": {
                            "type": "string"
                        },
                        "command": {
                            "type": "string"
                        }
                    },
                    "required": ["cwd", "command"]
                }),
            ]
        ]

    @classproperty
    def name(cls) -> str:
        return "DefaultTools"

    @property
    def list(self) -> list:
        return self._tools

    def call(self, tool_name: str, **args) -> str:
        if hasattr(self, tool_name): return getattr(self, tool_name)(**args)
        raise Exception(f"Tool {tool_name} not available")

    def _is_allowed(self, path: str, dirs: list) -> bool:
        abs_p = os.path.abspath(path)
        return any(abs_p.startswith(d) for d in dirs)

    def _check_access(self, path: str, mode: str):
        dirs = self._w_dirs if mode == 'w' else self._r_dirs
        if not self._is_allowed(path, dirs):
            raise Exception(f"{mode}-access denied to {path}")

    def read_file(self, path: str) -> str:
        self._check_access(path, 'r')
        with open(path, "r", encoding="utf-8") as f:
            return f.read()

    def write_file(self, path: str, content: str) -> str:
        self._check_access(path, 'w')
        os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        return f"success: wrote {path}"

    def create_file(self, path: str) -> str:
        self._check_access(path, 'w')
        with open(path, 'w') as f:
            pass
        return f"success: created {path}"

    def remove_file(self, path: str) -> str:
        self._check_access(path, 'w')
        os.remove(path)
        return f"success: removed {path}"

    def move_file(self, src: str, dst: str) -> str:
        self._check_access(src, 'w')
        self._check_access(dst, 'w')
        shutil.move(src, dst)
        return f"success: moved {src} to {dst}"

    def create_folder(self, path: str) -> str:
        self._check_access(path, 'w')
        os.makedirs(path, exist_ok=True)
        return f"success: created {path}"

    def remove_folder(self, path: str) -> str:
        self._check_access(path, 'w')
        shutil.rmtree(path)
        return f"success: removed {path}"

    def move_folder(self, src: str, dst: str) -> str:
        self._check_access(src, 'w')
        self._check_access(dst, 'w')
        shutil.move(src, dst)
        return f"success: moved {src} to {dst}"

    def list_available_shell_commands(self) -> str:
        return f"available: {self._r_shell}"

    def run_shell(self, cwd: str, command: str) -> str:
        self._check_access(cwd, 'r')
        res = subprocess.run(command, cwd=cwd, shell=True, capture_output=True)
        return (res.stdout + res.stderr).decode('utf-8', errors='replace')
