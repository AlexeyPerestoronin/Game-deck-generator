import os
import xai_sdk
import pathlib
import subprocess
import fnmatch

from . import i_tools

__all__ = [
    'FSTools',
]


class FSTools(i_tools.ITools):
    """Complex file system, git, and search tool set for AI agent"""

    def __init__(self, *, cwd: str | None = None, read_dirs: list[str], write_dirs: list[str], create_dirs: list[str]):
        self.__cwd = pathlib.Path(os.getcwd() if not cwd else cwd).absolute()
        self.__read_dirs = [pathlib.Path(dir).absolute() for dir in read_dirs]
        self.__write_dirs = [pathlib.Path(dir).absolute() for dir in write_dirs]
        self.__create_dirs = [pathlib.Path(dir).absolute() for dir in create_dirs]

        self.__tools = {
            FSTools.create_file.__name__:
            xai_sdk.chat.tool(
                name=FSTools.create_file.__name__,
                description="Create a new empty file.",
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
            FSTools.read_file.__name__:
            xai_sdk.chat.tool(
                name=FSTools.read_file.__name__,
                description="Read the entire content of a file.",
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
            FSTools.overwrite_file.__name__:
            xai_sdk.chat.tool(
                name=FSTools.overwrite_file.__name__,
                description="Completely overwrite the file contents.",
                parameters={
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        },
                        "content": {
                            "type": "string"
                        }
                    },
                    "required": ["path", "content"],
                },
            ),
            FSTools.search_and_replace.__name__:
            xai_sdk.chat.tool(
                name=FSTools.search_and_replace.__name__,
                description="Find a precise block of text inside a file and replace it with a new block.",
                parameters={
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string"
                        },
                        "old_content": {
                            "type": "string",
                            "description": "The exact block of code to find"
                        },
                        "new_content": {
                            "type": "string",
                            "description": "The block of code to replace it with"
                        }
                    },
                    "required": ["path", "old_content", "new_content"],
                },
            ),
            FSTools.search_text.__name__:
            xai_sdk.chat.tool(
                name=FSTools.search_text.__name__,
                description="Grep-like search for a specific text string across all allowed read directories.",
                parameters={
                    "type": "object",
                    "properties": {
                        "query": {
                            "type": "string",
                            "description": "Substring to find inside files"
                        },
                        "file_pattern": {
                            "type": "string",
                            "description": "Optional glob pattern like '*.py'"
                        }
                    },
                    "required": ["query"],
                },
            ),
            FSTools.git_status.__name__:
            xai_sdk.chat.tool(
                name=FSTools.git_status.__name__,
                description="Get current git status (modified files, untracked files).",
                parameters={
                    "type": "object",
                    "properties": {}
                },
            ),
            FSTools.git_diff.__name__:
            xai_sdk.chat.tool(
                name=FSTools.git_diff.__name__,
                description="Get git diff for modified files to review changes.",
                parameters={
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "Optional specific file path"
                        }
                    }
                },
            ),
        }

    @property
    def list(self) -> list:
        return list(self.__tools.values())

    def call(self, tool_name: str, **args) -> str:
        tool = getattr(self, tool_name, None)
        if tool and callable(tool):
            return tool(**args)
        raise Exception(f"calling tool is not available (list of available tools is {list(self.__tools.keys())})")

    def _resolve_path(self, path: str) -> pathlib.Path:
        p = pathlib.Path(path)
        if not p.is_absolute():
            return (self.__cwd / p).resolve()
        return p.resolve()

    def _is_path_safe(self, path: pathlib.Path, allowed_dirs: list[pathlib.Path]) -> bool:
        try:
            resolved_path = path.resolve()
            return any(resolved_path.is_relative_to(allowed) for allowed in allowed_dirs)
        except Exception:
            return False

    def _run_git_cmd(self, args: list[str]) -> str:
        """Безопасный запуск git-команд в контексте CWD"""
        if not self._is_path_safe(self.__cwd, self.__read_dirs):
            return "error: current working directory is not within allowed read directories"
        try:
            result = subprocess.run(["git"] + args, cwd=self.__cwd, capture_output=True, text=True, encoding="utf-8", timeout=10)
            if result.returncode != 0:
                return f"git error (code {result.returncode}): {result.stderr.strip()}"
            return result.stdout if result.stdout else "success (no output)"
        except Exception as e:
            return f"error running git command: {str(e)}"

    # --- 1. Поиск по коду ---

    def search_text(self, query: str, file_pattern: str = "*") -> str:
        """Ищет строку во всех файлах внутри разрешенных директорий чтения"""
        results = []
        for base_dir in self.__read_dirs:
            if not base_dir.exists() or not base_dir.is_dir():
                continue
            for root, _, files in os.walk(base_dir):
                for file in files:
                    if not fnmatch.fnmatch(file, file_pattern):
                        continue

                    full_path = pathlib.Path(root) / file
                    if not self._is_path_safe(full_path, self.__read_dirs):
                        continue

                    try:
                        # Читаем построчно для экономии памяти
                        with open(full_path, "r", encoding="utf-8", errors="ignore") as f:
                            for line_num, line in enumerate(f, 1):
                                if query in line:
                                    # Показываем относительный путь от CWD, если возможно
                                    try:
                                        display_path = full_path.relative_to(self.__cwd)
                                    except ValueError:
                                        display_path = full_path
                                    results.append(f"{display_path}:{line_num}: {line.strip()}")
                    except Exception:
                        pass  # Игнорируем бинарные или нечитаемые файлы

        if not results:
            return f"No matches found for query: '{query}'"
        return "\n".join(results)

    # --- 2. Улучшенное изменение файлов ---

    def search_and_replace(self, path: str, old_content: str, new_content: str) -> str:
        """Точечная замена блоков текста (заменяет капризный diff_file)"""
        abs_path = self._resolve_path(path)
        if not self._is_path_safe(abs_path, self.__write_dirs) or not self._is_path_safe(abs_path, self.__read_dirs):
            raise Exception(f"write/read-access outside the allowed directories is prohibited")

        if not abs_path.exists():
            return f"error: '{path}' does not exist"

        content = abs_path.read_text(encoding="utf-8")

        # Нормализуем переносы строк для стабильного поиска
        normalized_content = content.replace("\r\n", "\n")
        normalized_old = old_content.replace("\r\n", "\n")
        normalized_new = new_content.replace("\r\n", "\n")

        if normalized_old not in normalized_content:
            return "error: original code block (`old_content`) not found in the file exactly as specified"

        # Делаем ровно одно вхождение во избежание случайных множественных замен
        count = normalized_content.count(normalized_old)
        if count > 1:
            return f"error: original code block found {count} times. Make your `old_content` query more specific."

        updated_content = normalized_content.replace(normalized_old, normalized_new, 1)
        abs_path.write_text(updated_content, encoding="utf-8")
        return f"success: patched '{path}' successfully"

    # --- 3. Работа с Git ---

    def git_status(self) -> str:
        """Возвращает статус репозитория"""
        return self._run_git_cmd(["status", "-s"])

    def git_diff(self, path: str | None = None) -> str:
        """Возвращает изменения в файлах"""
        args = ["diff"]
        if path:
            abs_path = self._resolve_path(path)
            if not self._is_path_safe(abs_path, self.__read_dirs):
                raise Exception("access outside allowed read directories is prohibited")
            args.append(str(abs_path))
        return self._run_git_cmd(args)

    # --- Старые базовые CRUD методы (с фиксами) ---

    def is_file_exist(self, path: str) -> str:
        abs_path = self._resolve_path(path)
        if not self._is_path_safe(abs_path, self.__read_dirs): return "false"
        return "true" if abs_path.is_file() else "false"

    def create_file(self, path: str) -> str:
        abs_path = self._resolve_path(path)
        if not self._is_path_safe(abs_path, self.__create_dirs): raise Exception("prohibited")
