import os
import xai_sdk
import pathlib
import subprocess
import fnmatch
import shutil

from classproperties import classproperty

from typing import List, Tuple

from . import i_tools

__all__ = [
    'FileTools',
]


class FileTools(i_tools.ITools):
    """Complex file system, git, and search tool set for AI agent"""

    def __init__(self, cwd: str, allowed_dirs: List[str]):
        self._cwd = pathlib.Path(cwd).absolute()
        self._allowed_dirs = [pathlib.Path(dir).absolute() for dir in allowed_dirs]

        # yapf: disable
        self.__tools = {
            FileTools.create_file.__name__:
            xai_sdk.chat.tool(
                name=FileTools.create_file.__name__,
                description="Create a new empty file at the specified path.",
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

            FileTools.remove_file.__name__:
            xai_sdk.chat.tool(
                name=FileTools.remove_file.__name__,
                description="Remove a file from the file system.",
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

            FileTools.read_file.__name__:
            xai_sdk.chat.tool(
                name=FileTools.read_file.__name__,
                description="Read the entire text contents of a file.",
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

            FileTools.write_file.__name__:
            xai_sdk.chat.tool(
                name=FileTools.write_file.__name__,
                description="Write content to a file. Fails if the file already exists.",
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

            FileTools.overwrite_file.__name__:
            xai_sdk.chat.tool(
                name=FileTools.overwrite_file.__name__,
                description="Completely overwrite the existing file contents or create a new file if it does not exist.",
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
        }
        # yapf: enable

    # i_tools.ITools
    @classproperty
    def name(cls) -> str:
        return "FileTools"

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

    # --- 1. базовая работа с файловой системой ---

    def create_file(self, path: str) -> str:
        abs_path = pathlib.Path(path).absolute()
        if not self._check_path(abs_path, self._allowed_dirs):
            raise Exception("write-access outside the allowed directories is prohibited")
        if abs_path.exists():
            return f"error: '{path}' already exists"
        abs_path.parent.mkdir(parents=True, exist_ok=True)
        abs_path.touch()
        return f"success: created empty file '{path}'"

    def remove_file(self, path: str) -> str:
        abs_path = pathlib.Path(path).absolute()
        if not self._check_path(abs_path, self._allowed_dirs):
            raise Exception("write-access outside the allowed directories is prohibited")
        if not abs_path.exists() or not abs_path.is_file():
            return f"error: '{path}' is not a file or does not exist"
        abs_path.unlink()
        return f"success: removed '{path}'"

    def read_file(self, path: str) -> str:
        abs_path = pathlib.Path(path).absolute()
        if not self._check_path(abs_path, self._allowed_dirs):
            raise Exception("read-access outside the allowed directories is prohibited")
        if not abs_path.exists() or not abs_path.is_file():
            return f"error: '{path}' is not a file or does not exist"
        try:
            return abs_path.read_text(encoding="utf-8")
        except Exception as e:
            return f"error reading '{path}': {e}"

    def write_file(self, path: str, content: str) -> str:
        abs_path = pathlib.Path(path).absolute()
        if not self._check_path(abs_path, self._allowed_dirs):
            raise Exception("write-access outside the allowed directories is prohibited")
        if abs_path.exists():
            return f"error: '{path}' already exists"
        abs_path.parent.mkdir(parents=True, exist_ok=True)
        abs_path.write_text(content, encoding="utf-8")
        return f"success: wrote '{path}'"

    def overwrite_file(self, path: str, content: str) -> str:
        abs_path, error = self._check_path(path, self._allowed_dirs)
        if error:
            raise Exception(f"overwrite file '{path}' is prohibited outside allowed directories (allowed directories: {allowed_dir})")

        abs_path.parent.mkdir(parents=True, exist_ok=True)
        abs_path.write_text(content, encoding="utf-8")
        return f"success: overwrote '{path}'"

    # ---

    def _check_path(self, path: str, allowed_dirs: List[pathlib.Path]) -> Tuple[pathlib.Path | None, Exception | None]:
        """TODO: need to provide some comment"""
        try:
            abs_path = pathlib.Path(path).absolute()
            resolved_path = abs_path.resolve()
            if not any(resolved_path.is_relative_to(allowed) for allowed in allowed_dirs):
                return None, f"'{path}' is not allowed (allowed directories: {[str(dir.relative_to(self._cwd)) for dir in allowed_dirs]})"
            return abs_path, None
        except Exception as error:
            return None, f"exception: {error}"