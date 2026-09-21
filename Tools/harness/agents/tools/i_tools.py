import typing

from classproperties import classproperty

__all__ = [
    'ITools',
]


class ITools(typing.Protocol):
    """Protocol for tool sets exposed to the AI agent."""

    @classproperty
    def name(cls) -> str:
        ...

    @property
    def list(self) -> list:
        ...

    def call(self, tool_name: str, **args) -> str:
        ...
