import io
import os
import git
import shutil
import subprocess

from classproperties import classproperty

from . import i_tools

__all__ = ['DefaultTools']


class DefaultTools(i_tools.ITools):
    """Default tool set: shell execution (with confirm), read/write file (sandboxed)."""

    def __init__(self, safe_mode: bool, settings: dict):
        self._safe_mode = safe_mode
        self._command_execution_limit = settings.get("command-execution-limit", 60)
        self._w_shell = [c[2:] for c in settings.get("shell", []) if c.startswith("w:")]
        self._r_shell = [c[2:] for c in settings.get("shell", []) if c.startswith("r:")] + self._w_shell
        self._w_dirs = [os.path.abspath(d[2:]) for d in settings.get("dirs", []) if d.startswith("w:")]
        self._r_dirs = [os.path.abspath(d[2:]) for d in settings.get("dirs", []) if d.startswith("r:")] + self._w_dirs

        self._tools = [
            i_tools.Tool(n, d, p) for n, d, p in [
                (DefaultTools.read_file.__name__, "Прочитать содержимое файла.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                (DefaultTools.write_file.__name__, "Записать файл.", {
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
                (DefaultTools.patch_file.__name__, "Применить diff-патч к файлу.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        },
                        "patch": {
                            "type": "string"
                        }
                    },
                    "required": ["path", "patch"]
                }),
                (DefaultTools.create_file.__name__, "Создать файл.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                (DefaultTools.remove_file.__name__, "Удалить файл.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                (DefaultTools.move_file.__name__, "Переместить файл.", {
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
                (DefaultTools.list_folder.__name__, "Возвращает содержимое директории на заданную глубину (depth >= 0, где 0 - только корневое содержимое).", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                        },
                        "depth": {
                            "type": "string",
                        }
                    },
                    "required": ["path", "depth"]
                }),
                (DefaultTools.create_folder.__name__, "Создать папку.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                (DefaultTools.remove_folder.__name__, "Удалить папку.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                (DefaultTools.move_folder.__name__, "Переместить папку.", {
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
                (DefaultTools.list_available_shell_commands.__name__, "Список доступных shell команд.", {}),
                (DefaultTools.run_shell.__name__, "Запуск команды.", {
                    "type": "object",
                    "properties": {
                        "cwd": {
                            "type": "string"
                        },
                        "command": {
                            "type": "string"
                        },
                        "arguments": {
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

    def _check_access(self, path: str, mode: str, error: str | None = None):
        dirs = self._w_dirs if mode == 'w' else self._r_dirs
        if not self._is_allowed(path, dirs):
            raise Exception(error if error else f"{mode}-access denied to {path}")


    # text tools

    def patch_file(self, path: str, patch: str) -> str:
        def retarget_patch(patch: str, filename: str) -> str:
            # rewrite unified-diff headers so `git apply` touches only `filename`
            lines = []
            has_header = False
            for line in patch.splitlines():
                if line.startswith('--- '):
                    lines.append('--- /dev/null' if line.startswith('--- /dev/null') else f'--- a/{filename}')
                    has_header = True
                elif line.startswith('+++ '):
                    lines.append('+++ /dev/null' if line.startswith('+++ /dev/null') else f'+++ b/{filename}')
                elif line.startswith('diff --git '):
                    lines.append(f'diff --git a/{filename} b/{filename}')
                else:
                    lines.append(line)
            if not has_header:
                lines = [f'diff --git a/{filename} b/{filename}', f'--- a/{filename}', f'+++ b/{filename}'] + lines
            return '\n'.join(lines) + '\n'

        self._check_access(path, 'w')
        abs_path = os.path.abspath(path)
        payload = retarget_patch(patch, os.path.basename(abs_path))
        git.Git(os.path.dirname(abs_path)).apply(istream=io.BytesIO(payload.encode('utf-8')))
        return f"success: patched {path}"

    # file tools

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

    # folder tools

    def list_folder(self, path: str, depth: str) -> str:
        depth = int(depth)
        def walk_folder(path: str, depth: int) -> list:
            result = []
            if depth > 0:
                dirs = []
                files = []
                for name in sorted(os.listdir(path)):
                    full = os.path.join(path, name)
                    if os.path.isdir(full):
                        dirs.append(full)
                    else:
                        files.append(full)

                for folder in dirs:
                    result.append(folder)
                    result.extend(walk_folder(folder, depth - 1))
                result.extend(files)
            return result

        self._check_access(path, 'r')
        return "\n".join(walk_folder(path, depth))

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

    # command tools

    def list_available_shell_commands(self) -> str:
        return f"available: {self._r_shell}"

    def run_shell(self, cwd: str, command: str, arguments: str) -> str:
        if command not in self._r_shell: raise Exception(f"'{command}'-command is not available (available command list is {self._r_shell})")
        if command in self._w_shell: self._check_access(cwd, 'w', f"'{cwd}'-cwd is denied for '{command}'-command (allowed cwd for '{command}'-command is {self._w_dirs})")
        if command in self._r_shell: self._check_access(cwd, 'r', f"'{cwd}'-cwd is denied for '{command}'-command (allowed cwd for '{command}'-command is {self._r_dirs})")

        try:
            cmd = command + " " + arguments
            result = subprocess.run(
                cmd,
                cwd=cwd,
                shell=True,
                capture_output=True,
                text=False,
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
            raise Exception(f"execution of the '{cmd}' exceed the limit (available limit is {self._command_execution_limit}s)")
