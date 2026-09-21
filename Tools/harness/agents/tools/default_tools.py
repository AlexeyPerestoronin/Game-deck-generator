import os
import subprocess

from classproperties import classproperty

from . import i_tools

__all__ = [
    'DefaultTools',
]

class DefaultTools(i_tools.ITools):
    """Default tool set: shell execution (with confirm), read/write file (sandboxed to cwd)."""

    def __init__(self, safe_mode: bool, available_shell: list = None, available_read_dirs: list = None, available_write_dirs: list = None):
        self._safe_mode = safe_mode
        self._available_shell = available_shell or []
        self._available_read_dirs = available_read_dirs or []
        self._available_write_dirs = available_write_dirs or []

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

    def _is_path_allowed(self, abs_path: str, dirs: list) -> bool:
        """Return True if abs_path is inside one of the allowed dirs (simple sandbox check)."""
        for d in dirs:
            d_abs = os.path.abspath(d)
            if os.path.commonpath([abs_path, d_abs]) == d_abs:
                return True
        return False

    def read_file(self, path: str) -> str:
        abs_path = os.path.abspath(path)
        if not self._is_path_allowed(abs_path, self._available_read_dirs):
            raise Exception(f"read-access outside the allowed directories is prohibited (read allowed directories is {self._available_read_dirs})")
        if not os.path.exists(abs_path) or not os.path.isfile(abs_path):
            return f"error: '{path}' is not a file or does not exist"
        try:
            with open(abs_path, "r", encoding="utf-8") as f:
                return f.read()
        except Exception as e:
            return f"error reading '{path}': {e}"

    def write_file(self, path: str, content: str) -> str:
        abs_path = os.path.abspath(path)
        if not self._is_path_allowed(abs_path, self._available_write_dirs):
            raise Exception(f"write-access outside the allowed directories is prohibited (write allowed directories is {self._available_write_dirs})")
        if os.path.exists(abs_path):
            return f"error: '{path}' already exists"
        os.makedirs(os.path.dirname(abs_path), exist_ok=True)
        with open(abs_path, "w", encoding="utf-8") as f:
            f.write(content)
        return f"success: wrote '{path}'"

    def list_available_shell_commands(self) -> str:
        return f"available commands list {self._available_shell}"

    def run_shell(self, command: str) -> str:
        if self._available_shell:
            cmd = command.strip().split(maxsplit=1)[0] if command and command.strip() else ""
            if cmd not in self._available_shell:
                raise Exception(f"shell command '{cmd}' is not in the list of available commands {self._available_shell}")
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
