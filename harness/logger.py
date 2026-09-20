import typing
import pathlib

class Logger(typing.Protocol):
    """TODO: need to provide some comment"""

    def log_line(self, message: str = "") -> 'Logger':
        ...

class DoubleLogger(Logger):
    def __init__(self, log_file: pathlib.Path):
        self.__log_file = log_file
        if not self.__log_file.exists():
            open(self.__log_file, 'x')
        
    def log_line(self, message: str = "") -> 'Logger':
        if message[-1] != '\n':
            message += '\n'
        print(message)
        with open(self.__log_file, 'a') as file:
            file.write(message)
        return self