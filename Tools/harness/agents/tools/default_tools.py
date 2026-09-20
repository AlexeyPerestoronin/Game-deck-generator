import os
import xai_sdk
import subprocess

from . import i_tools

__all__ = [
    'DefaultTools',
]


class DefaultTools(i_tools.ITools):
    """Default tool set: shell execution (with confirm), read/write file (sandboxed to cwd)."""

    def __init__(self, safe_mode):
        self.__safe_mode = safe_mode
        self.__tools = {
            DefaultTools.run_shell.__name__:
            xai_sdk.chat.tool(
                name=DefaultTools.run_shell.__name__,
                description="Выполнить shell-команду.",
                parameters={
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string"
                        }
                    },
                    "required": ["command"],
                },
            ),
            DefaultTools.read_file.__name__:
            xai_sdk.chat.tool(
                name=DefaultTools.read_file.__name__,
                description="Прочитать содержимое текстового файла.",
                parameters={
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"],
                },
            ),
            DefaultTools.write_file.__name__:
            xai_sdk.chat.tool(
                name=DefaultTools.write_file.__name__,
                description="Записать или перезаписать файл.",
                parameters={
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        },
                        "content": {
                            "type": "string"
                        },
                    },
                    "required": ["path", "content"],
                },
            ),
        }

    @property
    def list(self) -> list:
        return self.__tools.values()

    def call(self, tool_name: str, **args) -> str:
        tool = getattr(self, tool_name, None)
        if tool and callable(tool):
            return tool(**args)
        raise Exception(f"calling tool is not available (list of available tools is {list(self.__tools.keys())})")

    # tools

    def _is_path_safe(self, path: str) -> bool:
        abs_path = os.path.abspath(path)
        cwd = os.path.abspath(os.getcwd())
        prefix = cwd if cwd.endswith(os.sep) else cwd + os.sep
        return abs_path.startswith(prefix)

    def run_shell(self, command: str) -> str:
        if self.__safe_mode:
            confirm = input(f"Execute next command: `{command}`? [y/N]: ").strip().lower()
            if confirm not in ("y", "yes"):
                raise Exception("the user has prohibited the execution of the command")
        try:
            result = subprocess.run(command, shell=True, capture_output=True, text=True, timeout=30)
            output = result.stdout or result.stderr
            return output if output else "(command finished without output)"
        except subprocess.TimeoutExpired:
            raise Exception("command execution time limit exceed (available limit is 30s)")

    def read_file(self, path: str) -> str:
        if not self._is_path_safe(path):
            raise Exception("access outside the working directory is prohibited")
        if not os.path.exists(path):
            raise Exception(f"file '{path}' is not found")
        with open(path, "r", encoding="utf-8") as f:
            return f.read()

    def write_file(self, path: str, content: str) -> str:
        if not self._is_path_safe(path):
            raise Exception("access outside the working directory is prohibited")
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        return f"Файл {path} успешно сохранён."
