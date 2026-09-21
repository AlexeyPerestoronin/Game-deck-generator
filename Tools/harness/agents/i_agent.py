import typing

__all__ = [
    'IAgent',
]


class IAgent(typing.Protocol):
    """Protocol for pluggable AI agents in the harness."""

    @property
    def name(cls) -> str:
        ...

    @property
    def tokens_limit(self) -> int:
        ...

    @property
    def conversation_id(self) -> str:
        ...

    @property
    def chat_id(self) -> str:
        ...

    @property
    def consumed_tokens(self) -> int:
        ...

    @property
    def consumed_usd(self) -> float:
        ...

    def iteration(self) -> bool:
        ...

    def finish(self):
        ...
