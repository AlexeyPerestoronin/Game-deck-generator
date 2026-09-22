import typing
import pathlib
from classproperties import classproperty

__all__ = [
    'IAgent',
]


class IAgent(typing.Protocol):
    """Protocol for pluggable AI agents in the harness."""

    @classproperty
    def vendor(cls) -> str:
        """TODO: need to provide some comment"""
        ...

    @property
    def model(self) -> str:
        """TODO: need to provide some comment"""
        ...

    def iteration(self) -> bool:
        """TODO: need to provide some comment"""
        ...

    def finish(self):
        """TODO: need to provide some comment"""
        ...

    def dump_session(self, dump_file: pathlib.Path):
        """TODO: need to provide some comment"""
        ...

    def reload_session(self, dump_file: pathlib.Path):
        """TODO: need to provide some comment"""
        ...
