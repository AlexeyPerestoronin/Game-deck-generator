import os
import xai_sdk
import pathlib

from . import i_tools

__all__ = [
    'FSTools',
]

class FSTools(i_tools.ITools):
    """Complex file system tool set for AI agent"""

    def __init__(self, *, cwd: str | None = None, read_dirs: list[str], write_dirs: list[str], create_dirs: list[str]):
        self.__cwd = pathlib.Path(os.getcwd() if not cwd else cwd).absolute()
        self.__read_dirs = [pathlib.Path(dir).absolute() for dir in read_dirs]
        self.__write_dirs = [pathlib.Path(dir).absolute() for dir in write_dirs]
        self.__create_dirs = [pathlib.Path(dir).absolute() for dir in create_dirs]

        self.__tools = {
            FSTools.create_file.__name__:
            xai_sdk.chat.tool(
                name=FSTools.read_file.__name__,
                description="create new file",
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
                description="read the file content",
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
                description="overwrite the file contents",
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

    def _is_path_safe(self, path: pathlib.Path, allowed_dirs: list[pathlib.Path]) -> bool:
        # TODO: need to implement
        ...

    # file tools

    def is_file_exist(self, path: str) -> str:
        # TODO: need to implement
        # функция должна возвращать результат проверки наличия файла по пути
        ...

    def create_file(self, path: str) -> str:
        abs_path = pathlib.Path(path).absolute()
        if not self._is_path_safe(abs_path, self.__create_dirs):
            raise Exception(f"create-file-access outside the next working directories {self.__create_dirs} is prohibited")
        if not abs_path.exists():
            abs_path.touch()
            return f"success: '{path}' file created"
        return f"success: '{path}' file already exist"

    def remove_file(self, path: str) -> str:
        # TODO: need to implement
        # функция должна удалять файл по переданному пути
        ...

    def read_file(self, path: str) -> str:
        abs_path = pathlib.Path(path).absolute()
        if not self._is_path_safe(abs_path, self.__read_dirs):
            raise Exception(f"read-file-access outside the next working directories {self.__read_dirs} is prohibited")
        if not abs_path.exists():
            return f"error: '{path}' is not exist"
        with open(abs_path, "r", encoding="utf-8") as f:
            return f.read()

    def overwrite_file(self, path: str, content: str) -> str:
        abs_path = pathlib.Path(path).absolute()
        if not self._is_path_safe(abs_path, self.__write_dirs):
            raise Exception(f"write-file-access outside the next working directories {self.__write_dirs} is prohibited")
        with open(path, "w", encoding="utf-8") as f:
            f.write(content)
        return f"success: '{path}' успешно сохранён."

    def diff_file(self, path: str, diff: str) -> str:
        # TODO: need to implement
        # функция должна применять к уже существующему файлу diff для точечного изменения его содержимого
        ...

    # directory tools

    def is_dir_exist(self, path: str) -> str:
        # TODO: need to implement
        # функция должна возвращать результат проверки наличия директории по пути
        ...

    def list_dir(self, path: str) -> str:
        # TODO: need to implement
        # функция должна возвращать в качестве ответа полный список всех дочерних элементов директории
        ...

    def create_dir(self, path: str) -> str:
        # TODO: need to implement
        # функция должна создавать директорию по переданному пути
        ...

    def remove_dir(self, path: str) -> str:
        # TODO: need to implement
        # функция должна удалять директорию по переданному пути
        ...