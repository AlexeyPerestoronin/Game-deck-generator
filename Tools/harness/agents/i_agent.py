import typing
from classproperties import classproperty

__all__ = [
    'IAgent',
]


class IAgent(typing.Protocol):
    """Protocol for pluggable AI agents in the harness."""

    @classproperty
    def vendor(cls) -> str:
        ...

    @property
    def model(self) -> str:
        ...

    def iteration(self) -> bool:
        ...

    def finish(self):
        ...
