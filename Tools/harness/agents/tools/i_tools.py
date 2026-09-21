import typing

__all__ = [
    'ITools',
]


class ITools(typing.Protocol):
    """Protocol for tool sets exposed to the AI agent."""

    @property
    def name(cls) -> str:
        ...

    @property
    def list(self) -> list:
        ...

    def call(self, tool_name: str, **args) -> str:
        ...
