import os
import subprocess

from classproperties import classproperty

from . import i_tools

__all__ = [
    'DefaultTools',
]

class DefaultTools(i_tools.ITools):
    """Default tool set: shell execution (with confirm), read/write file (sandboxed to cwd)."""

    def __init__(self, safe_mode: bool, available_shell: list, available_read_dirs: list, available_write_dirs: list):
        self._safe_mode = safe_mode
        self._available_shell = available_shell
        self._available_read_dirs = available_read_dirs
        self._available_write_dirs = available_write_dirs

        #yapf: disable
        self._tools = [
            i_tools.Tool(
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

            i_tools.Tool(
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

            i_tools.Tool(
                name=DefaultTools.list_available_shell_commands.__name__,
                description="Узнать список доступных shell команд",
                parameters={},
            ),

            i_tools.Tool(
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
        ]
        #yapf: enable

    # i_tools.ITools
    @classproperty
    def name(cls) -> str:
        return "DefaultTools"

    # i_tools.ITools
    @property
    def list(self) -> list:
        return self._tools

    # i_tools.ITools
    def call(self, tool_name: str, **args) -> str:
        for tool in self._tools:
            if tool.name == tool_name:
                return getattr(self, tool_name, None)(**args)
        raise Exception(f"calling tool is not available (list of available tools is {[tool.name for tool in self._tools]})")

    # tools

    def read_file(self, path: str) -> str:
        # TODO: надо реализовать чтение в файл с проверкой нахождения файла в разрешённой директории
        pass

    def write_file(self, path: str, content: str) -> str:
        # TODO: надо реализовать запись в файл с проверкой нахождения файла в разрешённой директории
        pass

    def list_available_shell_commands(self) -> str:
        return f"available commands list {self._available_shell}"

    def run_shell(self, command: str) -> str:
        # TODO: надо реализовать проверку, что вызываемая shell команда в списке доступных
        if self._safe_mode:
            confirm = input(f"Execute next command: `{command}`? [y/N]: ").strip().lower()
            if confirm not in ("y", "yes"):
                raise Exception("the user has prohibited the execution of the command")
        try:
            result = subprocess.run(command, shell=True, capture_output=True, text=True, timeout=30)
            output = result.stdout or result.stderr
            return output if output else "(command finished without output)"
        except subprocess.TimeoutExpired:
            raise Exception("command execution time limit exceed (available limit is 30s)")
