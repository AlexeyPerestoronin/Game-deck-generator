import os
import xai_sdk
import subprocess

from typing import Protocol

__all__ = ['Tools', 'DefaultToolsSet',]


class Tools(Protocol):

    @property
    def list(self) -> list:
        ...

    def call(self, tool_name: str, **args) -> str:
        ...


class DefaultToolsSet(Tools):
    """TODO: need to provide some comment"""

    def __init__(self):
        self.__tools = {
            DefaultToolsSet.run_shell.__name__:
            xai_sdk.chat.tool(
                name=DefaultToolsSet.run_shell.__name__,
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
            DefaultToolsSet.read_file.__name__:
            xai_sdk.chat.tool(
                name=DefaultToolsSet.read_file.__name__,
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
            DefaultToolsSet.write_file.__name__:
            xai_sdk.chat.tool(
                name=DefaultToolsSet.write_file.__name__,
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
        raise Exception(f"calling tool is'not available (list of available tools is [{self.__tools.keys()}])")

    # tools

    def run_shell(self, command: str) -> str:
        confirm = input(f"Execute next command: `{command}`? [y/N]: ").strip().lower()
        if confirm not in ("y", "yes"):
            raise Exception("the user has prohibited the execution of the command")
        try:
            result = subprocess.run(command,
                                    shell=True,
                                    capture_output=True,
                                    text=True,
                                    timeout=30)
            output = result.stdout or result.stderr
            return output if output else "(command finished without output)"
        except subprocess.TimeoutExpired:
            raise Exception("command execution time limit exceed (available limit is 30s)")

    def read_file(self, path: str) -> str:
        if not os.path.exists(path):
            raise Exception(f"file '{path}' is not found")
        if not os.path.abspath(path).startswith(os.getcwd()):
            raise Exception("access outside the working directory is prohibited")
        with open(path, "r", encoding="utf-8") as f:
            return f.read()

    def write_file(self, path: str, content: str) -> str:
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        return f"Файл {path} успешно сохранён."
