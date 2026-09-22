"""Agent protocol for the harness loop.

IAgent is the contract implemented by vendor-specific agents: one iteration
of the session, completion, and dump/reload of session state.
"""

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
        """Return the vendor identifier of this agent implementation."""
        ...

    @property
    def model(self) -> str:
        """Return the concrete model name of this agent instance."""
        ...
    
    @property
    def consumed_tokens(self) -> int:
        """Return the quantity of consumed token for current moment in the session."""
        ...

    def iteration(self) -> bool:
        """Run a single agent loop iteration.

        Sends the current user prompt or pending tool results to the model,
        logs the response, and executes tool calls when requested.

        Returns:
            True if the session is complete; False if another iteration is needed.
        """
        ...

    def finish(self):
        """Finalize the session and write usage statistics to the logger."""
        ...

    def dump_session(self, dump_file: pathlib.Path):
        """Serialize the current session state to ``dump_file``.

        Args:
            dump_file: Path to the snapshot file to write.
        """
        ...

    def reload_session(self, dump_file: pathlib.Path):
        """Restore session state previously written by ``dump_session``.

        Args:
            dump_file: Path to the snapshot file to read.

        Raises:
            OSError: If the dump file cannot be read.
            ValueError: If the dump file content is invalid.
        """
        ...
