import os
import xai_sdk
import pathlib
import subprocess
import fnmatch
import shutil

from . import i_tools

__all__ = [
    'FSTools',
]

class FSTools(i_tools.ITools):
    """Complex file system, git, and search tool set for AI agent"""

    def __init__(self, *, cwd: str | None = None, allowed_dirs: list[str]):
        self.__cwd = pathlib.Path(os.getcwd() if not cwd else cwd).absolute()
        self.__allowed_dirs = [pathlib.Path(dir).absolute() for dir in allowed_dirs]

        # TODO: проверить и добавить в правильном порядке
        self.__tools = {
            FSTools.create_file.__name__: xai_sdk.chat.tool(
                name=FSTools.create_file.__name__,
                description="Create a new empty file at the specified path.",
                parameters={
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"],
                },
            ),
            
            FSTools.remove_file.__name__: xai_sdk.chat.tool(
                name=FSTools.remove_file.__name__,
                description="Remove a file from the file system.",
                parameters={
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"],
                },
            ),
            FSTools.read_file.__name__: xai_sdk.chat.tool(
                name=FSTools.read_file.__name__,
                description="Read the entire text contents of a file.",
                parameters={
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"],
                },
            ),
            FSTools.write_file.__name__: xai_sdk.chat.tool(
                name=FSTools.write_file.__name__,
                description="Write content to a file. Fails if the file already exists.",
                parameters={
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "content": {"type": "string"}
                    },
                    "required": ["path", "content"],
                },
            ),
            FSTools.overwrite_file.__name__: xai_sdk.chat.tool(
                name=FSTools.overwrite_file.__name__,
                description="Completely overwrite the existing file contents or create a new file if it does not exist.",
                parameters={
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "content": {"type": "string"}
                    },
                    "required": ["path", "content"],
                },
            ),

            FSTools.create_dir.__name__: xai_sdk.chat.tool(
                name=FSTools.create_dir.__name__,
                description="Create a directory path including missing parent directories.",
                parameters={
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"],
                },
            ),
            FSTools.remove_dir.__name__: xai_sdk.chat.tool(
                name=FSTools.remove_dir.__name__,
                description="Recursively remove a directory and all of its contents.",
                parameters={
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"],
                },
            ),
            FSTools.list_dir.__name__: xai_sdk.chat.tool(
                name=FSTools.list_dir.__name__,
                description="List all child files and directories inside the specified directory.",
                parameters={
                    "type": "object",
                    "properties": {"path": {"type": "string"}},
                    "required": ["path"],
                },
            ),
            FSTools.search_text.__name__: xai_sdk.chat.tool(
                name=FSTools.search_text.__name__,
                description="Search for a specific substring within text files across allowed directories (like grep).",
                parameters={
                    "type": "object",
                    "properties": {
                        "query": {"type": "string", "description": "Substring to search for"},
                        "file_pattern": {"type": "string", "description": "Optional glob pattern like '*.py'"}
                    },
                    "required": ["query"],
                },
            ),
            FSTools.search_and_replace.__name__: xai_sdk.chat.tool(
                name=FSTools.search_and_replace.__name__,
                description="Find an exact unique code/text block and replace it with a new block.",
                parameters={
                    "type": "object",
                    "properties": {
                        "path": {"type": "string"},
                        "old_content": {"type": "string", "description": "The exact code block to find"},
                        "new_content": {"type": "string", "description": "The replacement block"}
                    },
                    "required": ["path", "old_content", "new_content"],
                },
            ),
            FSTools.git_status.__name__: xai_sdk.chat.tool(
                name=FSTools.git_status.__name__,
                description="Get short status of modified and untracked repository files.",
                parameters={"type": "object", "properties": {}},
            ),
            FSTools.git_diff.__name__: xai_sdk.chat.tool(
                name=FSTools.git_diff.__name__,
                description="Show changes in the working directory compared to the index.",
                parameters={
                    "type": "object",
                    "properties": {"path": {"type": "string", "description": "Optional file path to filter diff"}}
                },
            ),
        }

    # i_tools.ITools
    @property
    def list(self) -> list:
        return list(self.__tools.values())

    # i_tools.ITools
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

    # --- 1. базовая работа с файловой системой ---

    def create_file(self, path: str) -> str:
        # TODO: need to implement
    
    def remove_file(self, path: str) -> str:
        # TODO: need to implement
    
    def read_file(self, path: str) -> str:
        # TODO: need to implement

    def write_file(self, path: str) -> str:
        # TODO: need to implement
        
    def overwrite_file(self, path: str) -> str:
        # TODO: need to implement

    def create_dir(self, path: str) -> str:
        # TODO: need to implement
    
    def remove_dir(self, path: str) -> str:
        # TODO: need to implement
    
    def list_dir(self, path: str) -> str:
        # TODO: need to implement

    # --- 2. Поиск по коду ---

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

    # --- 3. Улучшенное изменение файлов ---

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

    # --- 4. Работа с Git ---

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