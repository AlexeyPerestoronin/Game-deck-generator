from typing import Protocol, List

from classproperties import classproperty

__all__ = [
    'Tool',
    'ITools',
]


class Tool:
    """Concrete descriptor for a tool exposed to AI agent (provides .name, .description, .parameters)."""

    def __init__(self, name: str, description: str, parameters: dict):
        self._name = name
        self._description = description
        self._parameters = parameters or {}

    @property
    def name(self) -> str:
        return self._name

    @property
    def description(self) -> str:
        return self._description

    @property
    def parameters(self) -> dict:
        return self._parameters


class ITools(Protocol):
    """Protocol for tool sets exposed to the AI agent."""

    @classproperty
    def name(cls) -> str:
        ...

    @property
    def list(self) -> List[Tool]:
        ...

    def call(self, tool_name: str, **args) -> str:
        ...
