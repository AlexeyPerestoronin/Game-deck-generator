import os
import shutil
import pathlib
import subprocess
import tempfile

from typing import Tuple
from classproperties import classproperty

from . import i_tools
from ... import logger

__all__ = ['DefaultTools']


# NOTE: при внесении изменений в список доступных команд, необходимо обновлять справку tool_help.md до актуального состояния.
class DefaultTools(i_tools.ITools):
    """Default tool set: shell execution (with confirm), read/write file (sandboxed)."""

    def __init__(self, safe_mode: bool, logger: logger.ILogger, settings: dict):
        self._safe_mode = safe_mode
        self._logger = logger

        self._cwd = settings["cwd"]
        self._temp_dir = settings["temp-dir"]
        self._available_file_extension = settings["available-file-extensions"]
        self._command_execution_limit = settings.get("command-execution-limit", 60)
        self._invoke_execution_limit = settings.get("invoke-execution-limit", 120)

        # r = read
        # w = write

        self._w_shell = [c[2:] for c in settings.get("shell", []) if c.startswith("w:")]
        self._r_shell = [c[2:] for c in settings.get("shell", []) if c.startswith("r:")] + self._w_shell

        self._w_dirs = [d[2:] for d in settings.get("dirs", []) if d.startswith("w:")]
        self._r_dirs = [d[2:] for d in settings.get("dirs", []) if d.startswith("r:")] + self._w_dirs

        # точечный доступ к файлам вне разрешённых директорий (_w_files также входят в _r_files)
        self._w_files = [d[2:] for d in settings.get("files", []) if d.startswith("w:")]
        self._r_files = [d[2:] for d in settings.get("files", []) if d.startswith("r:")] + self._w_files

        (pathlib.Path(self._cwd) / pathlib.Path(self._temp_dir)).mkdir(parents=True, exist_ok=True)

        self._tools = [
            i_tools.Tool(n, d, p) for n, d, p in [
                (DefaultTools.list_available_file_extension.__name__, "Список доступных расширений файлов.", {}),
                (DefaultTools.list_available_r_dir.__name__, "Список доступных директорий для чтения.", {}),
                (DefaultTools.list_available_w_dir.__name__, "Список доступных директорий для записи.", {}),
                (DefaultTools.list_available_r_file.__name__, "Список доступных файлов для чтения.", {}),
                (DefaultTools.list_available_w_file.__name__, "Список доступных файлов для записи.", {}),
                (DefaultTools.list_available_shell_commands.__name__, "Список доступных shell команд.", {}),
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
                (DefaultTools.is_file_under_git.__name__, "Проверить, находится ли файл под контролем git.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                (DefaultTools.apply_diff_patch.__name__, "Применить diff-патч к файлам.", {
                    "type": "object",
                    "properties": {
                        "patch": {
                            "type": "string"
                        }
                    },
                    "required": ["patch"]
                }),
                (DefaultTools.get_file_diff.__name__, "Получить diff для целевого файла.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
                }),
                (DefaultTools.discard_file_changes.__name__, "Отменить изменения файла после HEAD.", {
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        }
                    },
                    "required": ["path"]
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
                (DefaultTools.run_invoke.__name__, "Запуск команды invoke.", {
                    "type": "object",
                    "properties": {
                        "command": {
                            "type": "string"
                        }
                    },
                    "required": ["command"]
                }),
                (DefaultTools.ask_user.__name__, "Задать вопрос пользователю и получить ответ.", {
                    "type": "object",
                    "properties": {
                        "question": {
                            "type": "string"
                        }
                    },
                    "required": ["question"]
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
        if not hasattr(self, tool_name):
            raise Exception(f"tool '{tool_name}' not available")
        result = getattr(self, tool_name)(**args)
        if tool_name == DefaultTools.apply_diff_patch.__name__:
            tmp = result[1]
            self._logger.log_line(f"Apply patch {tmp}")
            result = result[0]
        elif tool_name == DefaultTools.read_file.__name__:
            self._logger.log_line(f"Read {len(result)}symbols")
        elif tool_name == DefaultTools.write_file.__name__:
            self._logger.log_line(f"Write {len(result)}symbols")
        else:
            self._logger.log_line('Result:').log_line("```").log_line(result).log_line("```")
        return result

    def _is_allowed(self, path: str, dirs: list, files: list) -> bool:
        # директории — по префиксу пути, файлы — только точное совпадение
        abs_p = os.path.abspath(path)
        return any(abs_p.startswith(os.path.abspath(d)) for d in dirs) or any(abs_p == os.path.abspath(f) for f in files)

    def _check_access(self, path: str, mode: str, error: str | None = None):
        dirs = self._w_dirs if mode == 'w' else self._r_dirs
        files = self._w_files if mode == 'w' else self._r_files
        if not self._is_allowed(path, dirs, files):
            raise Exception(
                error if error else
                f"{mode}-access denied to {path} → unavailable file location (list of available directories for {mode}-access is {dirs}; list of available files for {mode}-access is {files})"
            )

    def _check_extensions(self, path: str, mode: str, error: str | None = None):
        file_extension = pathlib.Path(path).suffix
        if len(self._available_file_extension) != 0:
            if file_extension not in self._available_file_extension:
                raise Exception(error if error else
                                f"{mode}-access denied to {path} → unavailable file extension {file_extension} (list of available extensions is {self._available_file_extension})")

    def _run_git(self, args: list, cwd: str, stdin: str | None = None):
        # системный git с захватом вывода; stdin используется для git apply
        return subprocess.run(
            ["git", *args],
            cwd=cwd,
            input=stdin,
            capture_output=True,
            encoding="utf-8",
            errors="replace",
            timeout=self._command_execution_limit,
        )

    def _git_toplevel(self, cwd: str) -> str:
        # корень git-репозитория для указанного cwd
        result = self._run_git(["rev-parse", "--show-toplevel"], cwd)
        if result.returncode != 0:
            error_text = (result.stderr or result.stdout or "").strip()
            raise Exception(error_text or "not a git repository")
        return os.path.abspath(result.stdout.strip())

    def _extract_patch_target_paths(self, patch: str) -> list:
        # цели unified-diff / git diff: пути относительно корня репозитория
        paths = []
        seen = set()

        def add(raw: str):
            raw = raw.strip().strip('"')
            if raw.startswith("a/") or raw.startswith("b/"):
                raw = raw[2:]
            if not raw or raw == "/dev/null" or raw in seen:
                return
            seen.add(raw)
            paths.append(raw)

        for line in patch.splitlines():
            if line.startswith("diff --git "):
                parts = line.split()
                if len(parts) >= 4:
                    add(parts[2])
                    add(parts[3])
            elif line.startswith("--- ") or line.startswith("+++ "):
                add(line[4:].split("\t", 1)[0])
        return paths

    def _git_path_cwd(self, path: str) -> tuple:
        # (basename, directory) для git-команд в каталоге файла
        abs_path = os.path.abspath(path)
        return os.path.basename(abs_path), os.path.dirname(abs_path) or "."

    # help

    def verbosity_help(self) -> str:
        help_path = pathlib.Path(__file__).parent / "tool_help.md"
        try:
            with open(help_path, "r", encoding="utf-8") as f:
                return f.read()
        except Exception as error:
            raise Exception(f"cannot load verbosity help → {error}")

    def list_available_file_extension(self) -> str:
        return f"available: {self._available_file_extension if len(self._available_file_extension) > 0 else 'all'}"

    def list_available_r_dir(self) -> str:
        return f"available: {self._r_dirs}"

    def list_available_w_dir(self) -> str:
        return f"available: {self._w_dirs}"

    def list_available_r_file(self) -> str:
        return f"available: {self._r_files}"

    def list_available_w_file(self) -> str:
        return f"available: {self._w_files}"

    def list_available_shell_commands(self) -> str:
        return f"available: {self._r_shell}"

    # text tools

    def is_file_under_git(self, path: str) -> str:
        try:
            self._check_access(path, 'r')
            # Use the path as-is (repo-relative) and main cwd so that produced diffs contain full paths.
            # This ensures apply_diff_patch (which runs git apply from _cwd) sees matching paths in patch.
            result = self._run_git(["ls-files", "--error-unmatch", "--", path], self._cwd)
            if result.returncode == 0:
                return f"file '{path}' is under git"
            return f"file '{path}' is not under git"
        except Exception as error:
            raise Exception(f"cannot check if file '{path}' is under git → {error}")

    def apply_diff_patch(self, patch: str) -> Tuple[str, str]:
        try:
            if not patch or not str(patch).strip():
                raise Exception("invalid or empty patch format")
            targets = self._extract_patch_target_paths(patch)
            if not targets:
                raise Exception("invalid or empty patch format")
            for rel_path in targets:
                self._check_access(rel_path, 'w')

            # Пишем патч во временный файл — это значительно надёжнее stdin
            # (особенно с кириллицей, многострочными файлами и на Windows).
            # Гарантируем завершающий \n, иначе git apply может счесть патч повреждённым
            # (corrupt patch), если входная строка patch не оканчивается на перевод строки.
            fd, tmp = tempfile.mkstemp(suffix=".patch", dir=self._temp_dir)
            patch_to_write = patch if patch.endswith("\n") else (patch + "\n" if patch else patch)
            with os.fdopen(fd, "w", encoding="utf-8", newline="\n") as f:
                f.write(patch_to_write)
            result = self._run_git(["apply", "--ignore-whitespace", "--ignore-space-change", "--recount", tmp], self._cwd)

            if result.returncode != 0:
                error_text = (result.stderr or result.stdout or "").strip()
                raise Exception(error_text or f"git apply failed with code {result.returncode}")
            return f"successfully applied patch to {', '.join(targets)}", tmp
        except Exception as error:
            raise Exception(f"cannot apply diff patch → {error}")

    def get_file_diff(self, path: str) -> str:
        try:
            self._check_access(path, 'r')
            # Use full path + main _cwd so diff contains correct repo-relative paths usable by apply_diff_patch.
            result = self._run_git(["diff", "HEAD", "--", path], self._cwd)
            if result.returncode != 0:
                error_text = (result.stderr or result.stdout or "").strip()
                raise Exception(error_text or f"git diff failed with code {result.returncode}")
            return result.stdout
        except Exception as error:
            raise Exception(f"cannot get diff for '{path}' → {error}")

    def discard_file_changes(self, path: str) -> str:
        try:
            self._check_access(path, 'w')
            # Use full path + main _cwd so checkout works for deep paths and matches patch path expectations.
            result = self._run_git(["checkout", "HEAD", "--", path], self._cwd)
            if result.returncode != 0:
                error_text = (result.stderr or result.stdout or "").strip()
                raise Exception(error_text or f"git checkout failed with code {result.returncode}")
            return f"successfully discarded changes in '{path}'"
        except Exception as error:
            raise Exception(f"cannot discard changes in '{path}' → {error}")

    # file tools

    def read_file(self, path: str) -> str:
        try:
            self._check_access(path, 'r')
            self._check_extensions(path, 'r')
            with open(path, "r", encoding="utf-8") as f:
                return f.read()
        except Exception as error:
            raise Exception(f"cannot read file '{path}' → {error}")

    def write_file(self, path: str, content: str) -> str:
        try:
            self._check_access(path, 'w')
            self._check_extensions(path, 'w')
            os.makedirs(os.path.dirname(os.path.abspath(path)), exist_ok=True)
            with open(path, "w", encoding="utf-8") as f:
                f.write(content)
            return f"success: wrote {path}"
        except Exception as error:
            raise Exception(f"cannot write file '{path}' → {error}")

    def create_file(self, path: str) -> str:
        try:
            self._check_access(path, 'w')
            self._check_extensions(path, 'w')
            with open(path, 'w') as f:
                pass
            return f"success: created {path}"
        except Exception as error:
            raise Exception(f"cannot create file '{path}' → {error}")

    def remove_file(self, path: str) -> str:
        try:
            self._check_access(path, 'w')
            os.remove(path)
            return f"success: removed {path}"
        except Exception as error:
            raise Exception(f"cannot remove file '{path}' → {error}")

    def move_file(self, src: str, dst: str) -> str:
        try:
            self._check_access(src, 'w')
            self._check_extensions(src, 'w')
            self._check_access(dst, 'w')
            self._check_extensions(dst, 'w')
            shutil.move(src, dst)
            return f"success: moved {src} to {dst}"
        except Exception as error:
            raise Exception(f"cannot move file '{str}' to {dst} → {error}")

    # folder tools

    def list_folder(self, path: str, depth: str) -> str:
        try:

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

            depth = int(depth)
            self._check_access(path, 'r')
            return "\n".join(walk_folder(path, depth))
        except Exception as error:
            raise Exception(f"cannot list folder '{path}' → {error}")

    def create_folder(self, path: str) -> str:
        try:
            self._check_access(path, 'w')
            os.makedirs(path, exist_ok=True)
            return f"success: created {path}"
        except Exception as error:
            raise Exception(f"cannot create folder '{path}' → {error}")

    def remove_folder(self, path: str) -> str:
        try:
            self._check_access(path, 'w')
            shutil.rmtree(path)
            return f"success: removed {path}"
        except Exception as error:
            raise Exception(f"cannot remove folder '{path}' → {error}")

    def move_folder(self, src: str, dst: str) -> str:
        try:
            self._check_access(src, 'w')
            self._check_access(dst, 'w')
            shutil.move(src, dst)
            return f"success: moved {src} to {dst}"
        except Exception as error:
            raise Exception(f"cannot move folder '{src}' to '{dst}' → {error}")

    # command tools

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

    def run_invoke(self, command: str) -> str:
        try:
            # invoke с переданными аргументами в рабочей директории агента
            cmd = f"{self._cwd}/.venv/Scripts/python.exe -m invoke {command}"
            result = subprocess.run(
                cmd,
                cwd=self._cwd,
                shell=True,
                capture_output=True,
                text=False,
                timeout=self._invoke_execution_limit,
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
            raise Exception(f"execution of the '{cmd}' exceed the limit (available limit is {self._invoke_execution_limit}s)")

    # user communication

    def ask_user(self, question: str) -> str:
        try:
            return input(f"{question}\n")
        except Exception as error:
            raise Exception(f"cannot ask user → {error}")

    def request_read_access_for(self, reason: str, path: str) -> str:
        # TODO: need to implement
        # запрос у пользователя получения read–доступа к целевой папке/файлу с объяснением причины
        # если запрос удовлетворён, то целевой объект добавляется в соответствующий список
        # если запрос отклонён, то запрашивается input с котором будет объяснена причина отказа
        ...

    def request_write_access_for(self, reason: str, path: str) -> str:
        # TODO: need to implement
        # запрос у пользователя получения write–доступа к целевой папке/файлу с объяснением причины
        # если запрос удовлетворён, то целевой объект добавляется в соответствующий список
        # если запрос отклонён, то запрашивается input с котором будет объяснена причина отказа
        ...

    def request_command_shell_execution(self, reason: str, command: str, cwd: str) -> str:
        # TODO: need to implement
        # запрос у пользователя исполнения команды с объяснением причины
        # если запрос удовлетворён, то команда выполняется и её результаты возвращаются в качестве ответа
        # если запрос отклонён, то запрашивается input с котором будет объяснена причина отказа
        ...
