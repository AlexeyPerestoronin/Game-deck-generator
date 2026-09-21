import typing
from classproperties import classproperty

__all__ = [
    'IAgent',
]


class IAgent(typing.Protocol):
    """Protocol for pluggable AI agents in the harness."""

    @classproperty
    def name(cls) -> str:
        ...

    @property
    def tokens_limit(self) -> int:
        ...

    @property
    def consumed_tokens(self) -> int:
        ...

    def iteration(self) -> bool:
        ...

    def finish(self):
        ...
