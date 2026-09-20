import datetime

from . import agents, logger

__all__ = [
    'AgentLoop',
]

class AgentLoop:
    """Drives the agent through iterations, enforcing token and iteration limits."""

    def __init__(self, safe_mode: bool, iteration_limit: int, agent: agents.IAgent, logger: logger.ILogger):
        self.__safe_mode = safe_mode
        self.__iteration_limit = iteration_limit
        self.__iteration = 0
        self.__agent = agent
        self.__logger = logger

    def check_session_token_limit(self) -> bool:
        """Return True if within limit (or user approved over limit)."""
        limit = self.__agent.tokens_limit
        consumed = self.__agent.consumed_tokens
        if consumed > limit:
            message = f"⚠️ tokens limit exceed ({consumed} > {limit})"
            self.__logger.log_line(message)
            if self.__safe_mode:
                user_decision = input(f"{message} → resume execution? [y/N]: ").strip().lower()
                return user_decision in ("y", "yes")
        return True

    def check_session_iteration_limit(self) -> bool:
        """Return True if within limit (or user approved over limit)."""
        limit = self.__iteration_limit
        consumed = self.__iteration
        if consumed > limit:
            message = f"⚠️ iteration limit exceed ({consumed} > {limit})"
            self.__logger.log_line(message)
            if self.__safe_mode:
                user_decision = input(f"{message} → resume execution? [y/N]: ").strip().lower()
                return user_decision in ("y", "yes")
        return True

    def start(self):
        agent_name = f"{self.__agent.name}-AI-agent"
        self.__logger\
            .log_line(f"# Agent-loop session:")\
            .log_line(f"- agent: {agent_name}")\
            .log_line(f"- iteration limit: {self.__iteration_limit}")\
            .log_line(f"- tokens limit: {self.__agent.tokens_limit}")\
            .log_line(f"- start time: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M')}")\
            .log_line()\
            .log_line("# Iterations:")\

        while True:
            self.__iteration += 1

            self.__logger.\
                log_line("")\
               .log_line(f"## Iteration №{self.__iteration}:")

            if not self.check_session_token_limit():
                raise Exception("interrupt loop: token limit exceed")

            if not self.check_session_iteration_limit():
                raise Exception("interrupt loop: iteration limit exceed")

            if self.__agent.iteration():
                self.__logger.log_line(f"# Session results:")
                self.__agent.finish()
                break
