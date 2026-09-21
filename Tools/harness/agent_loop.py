import os
import datetime

from . import agents, logger

__all__ = [
    'AgentLoop',
]


class AgentLoop:
    """Drives the agent through iterations, enforcing token and iteration limits."""

    def __init__(self, safe_mode: bool, logger: logger.ILogger, settings):
        self._safe_mode = safe_mode
        self._logger = logger
        self._settings = settings

        self._iteration = 0
        self._iteration_limit = self._settings["iteration-limit"]

        self.__tools = agents.tools.DefaultTools(
            self._safe_mode,
            self._settings["tools"]["available-shell"],
            self._settings["tools"]["available-read-dirs"],
            self._settings["tools"]["available-write-dirs"],
        )

        prompt = self._settings["prompt"]
        requested_vendor = self._settings["vendor"]
        requested_model = self._settings["model"]
        if requested_vendor == agents.Grok.name:
            self.__agent = agents.Grok(prompt, self.__tools, self._logger, self._settings.get["token-limit"])
        elif requested_vendor == agents.GoogleAI.name:
            model_specification = agents.GoogleAIStudioModelsSpecifications.from_str(requested_model)
            self.__agent = agents.GoogleAI(prompt, self.__tools, self._logger, model_specification)
        else:
            raise Exception("unexpected model of agent")

    def check_session_token_limit(self) -> bool:
        """Return True if within limit (or user approved over limit)."""
        limit = self.__agent.tokens_limit
        consumed = self.__agent.consumed_tokens
        if consumed > limit:
            message = f"⚠️ tokens limit exceed ({consumed} > {limit})"
            self._logger.log_line(message)
            if self._safe_mode:
                user_decision = input(f"{message} → resume execution? [y/N]: ").strip().lower()
                return user_decision in ("y", "yes")
        return True

    def check_session_iteration_limit(self) -> bool:
        """Return True if within limit (or user approved over limit)."""
        limit = self._iteration_limit
        consumed = self._iteration
        if consumed > limit:
            message = f"⚠️ iteration limit exceed ({consumed} > {limit})"
            self._logger.log_line(message)
            if self._safe_mode:
                user_decision = input(f"{message} → resume execution? [y/N]: ").strip().lower()
                return user_decision in ("y", "yes")
        return True

    def start(self):
        agent_name = f"{self.__agent.name}-AI-agent"
        self._logger\
            .log_line(f"# Agent-loop session:")\
            .log_line(f"- agent: {agent_name}")\
            .log_line(f"- iteration limit: {self._iteration_limit}")\
            .log_line(f"- tokens limit: {self.__agent.tokens_limit}")\
            .log_line(f"- start time: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M')}")\
            .log_line()\
            .log_line("# Iterations:")\

        while True:
            self._iteration += 1

            self._logger.\
                log_line("")\
               .log_line(f"## Iteration №{self._iteration}:")

            if not self.check_session_token_limit():
                raise Exception("interrupt loop: token limit exceed")

            if not self.check_session_iteration_limit():
                raise Exception("interrupt loop: iteration limit exceed")

            if self.__agent.iteration():
                self._logger.log_line(f"# Session results:")
                self.__agent.finish()
                break
