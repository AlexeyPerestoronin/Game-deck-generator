import os
import subprocess

from classproperties import classproperty

from . import i_tools

__all__ = [
    'DefaultTools',
]


class DefaultTools(i_tools.ITools):
    """Default tool set: shell execution (with confirm), read/write file (sandboxed to cwd)."""

    def __init__(self, safe_mode: bool, settings: dir):
        self._safe_mode = safe_mode

        self._command_execution_limit = settings.get("command-execution-limit", 30)
        self._w_shell = [command[2:] for command in settings["shell"] if command[:2] == "w:"]
        self._r_shell = [command[2:] for command in settings["shell"] if command[:2] == "r:"]
        self._r_shell.extend(self._w_shell)

        self._w_dirs = [command[2:] for command in settings["dirs"] if command[:2] == "w:"]
        self._r_dirs = [command[2:] for command in settings["dirs"] if command[:2] == "r:"]
        self._r_dirs.extend(self._w_dirs)

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
                description="Выполнить shell-команду из указанной директории.",
                parameters={
                    "type": "object",
                    "properties": {
                        "cwd": {
                            "type": "string"
                        },
                        "command": {
                            "type": "string"
                        }
                    },
                    "required": ["cwd", "command"],
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

    # file work
    
    def create_file(self, path: str) -> str:
        # TODO: need to implement
        ...

    def remove_file(self, path: str) -> str:
        # TODO: need to implement
        ...

    def move_file(self, path: str) -> str:
        # TODO: need to implement
        ...

    def read_file(self, path: str) -> str:
        abs_path = os.path.abspath(path)
        if not self._is_path_allowed(abs_path, self._r_dirs):
            raise Exception(f"read-access outside the allowed directories is prohibited (read-access directories is {self._r_dirs})")
        if not os.path.exists(abs_path) or not os.path.isfile(abs_path):
            return f"error: '{path}' is not a file or does not exist"
        try:
            with open(abs_path, "r", encoding="utf-8") as f:
                return f.read()
        except Exception as e:
            return f"error reading '{path}': {e}"

    def write_file(self, path: str, content: str) -> str:
        abs_path = os.path.abspath(path)
        if not self._is_path_allowed(abs_path, self._w_dirs):
            raise Exception(f"write-access outside the allowed directories is prohibited (write-access directories is {self._w_dirs})")
        if os.path.exists(abs_path):
            return f"error: '{path}' already exists"
        os.makedirs(os.path.dirname(abs_path), exist_ok=True)
        with open(abs_path, "w", encoding="utf-8") as f:
            f.write(content)
        return f"success: wrote '{path}'"

    # folder work

    def create_folder(self, path: str) -> str:
        # TODO: need to implement
        ...

    def remove_folder(self, path: str) -> str:
        # TODO: need to implement
        ...

    def move_folder(self, path: str) -> str:
        # TODO: need to implement
        ...

    # shell

    def list_available_shell_commands(self) -> str:
        return f"available commands list {self._r_shell}"

    def run_shell(self, cwd: str, command: str) -> str:
        if self._r_shell:
            cmd = command.strip().split(maxsplit=1)[0] if command and command.strip() else ""
            if cmd not in self._r_shell:
                raise Exception(f"shell command '{cmd}' is not in the list of available commands {self._r_shell}")

        abs_cwd = os.path.abspath(cwd)

        if command in self._w_shell:
            if not self._is_path_allowed(abs_cwd, self._w_dirs):
                raise Exception(f"'cwd'-parameter for '{command}'-command should be one of {self._w_dirs}")

        if not self._is_path_allowed(abs_cwd, self._r_dirs):
            raise Exception(f"'cwd'-parameter for '{command}'-command should be one of {self._r_dirs}")

        if self._safe_mode:
            confirm = input(f"Execute next command: `{command}`? [y/N]: ").strip().lower()
            if confirm not in ("y", "yes"):
                raise Exception(f"the user has prohibited the execution of the '{command}'-command")
        try:
            result = subprocess.run(
                command,
                cwd=cwd,
                shell=True,
                capture_output=True,
                text=False,  # returning raw bites
                timeout=self._command_execution_limit,
            )
            raw_output = result.stdout or result.stderr
            if raw_output:
                    encodings_to_try = ['utf-8', 'oem', 'cp1251']
                    for encoding in encodings_to_try:
                        try:
                            return raw_output.decode(encoding)
                        except UnicodeDecodeError:
                            continue
                    return raw_output.decode('utf-8', errors='replace')

            return "(command finished without output)"
        except subprocess.TimeoutExpired:
            raise Exception(f"execution of the '{command}'-command exceed the limit (available limit is {self._command_execution_limit}s)")
