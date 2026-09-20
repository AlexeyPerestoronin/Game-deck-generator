import typing
import pathlib


class Logger(typing.Protocol):
    """Logging interface for agent harness (stdout + file)."""

    def log_line(self, message: str = "") -> 'Logger':
        ...


class DoubleLogger(Logger):
    """Logger implementation that duplicates output to console and log file."""

    def __init__(self, log_file: pathlib.Path):
        self.__log_file = log_file
        self.__log_file.touch(exist_ok=True)

    def log_line(self, message: str = "") -> 'Logger':
        if not message.endswith("\n"):
            message += "\n"
        print(message, end="")
        with open(self.__log_file, "a", encoding="utf-8") as file:
            file.write(message)
        return self
