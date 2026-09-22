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

        self.__tools = agents.tools.DefaultTools(self._safe_mode, self._settings["tool-settings"])

        requested_vendor = self._settings["vendor"]
        requested_model = self._settings["model"]
        if requested_vendor == agents.SpaceXAI.vendor:
            model_specification = agents.SpaceXModels.from_str(requested_model)
            self._agent = agents.SpaceXAI(self.__tools, self._logger, model_specification)
        elif requested_vendor == agents.GoogleAI.vendor:
            model_specification = agents.GoogleModels.from_str(requested_model)
            self._agent = agents.GoogleAI(self.__tools, self._logger, model_specification)
        else:
            raise Exception("unexpected model of agent")

    def check_session_token_limit(self) -> bool:
        """Return True if within limit (or user approved over limit)."""
        limit = self._agent.tokens_limit
        consumed = self._agent.consumed_tokens
        if limit != -1 and consumed > limit:
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

    def start(self, prompt: str):
        self._logger\
            .log_line(f"# Agent-loop session:")\
            .log_line(f"- agent: {self._agent.vendor} {self._agent.model}")\
            .log_line(f"- iteration limit: {self._iteration_limit}")\
            .log_line(f"- tokens limit: {self._agent.tokens_limit}")\
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

            if self._agent.iteration(prompt):
                self._logger.log_line(f"# Session results:")
                self._agent.finish()
                break
            # one prompt per loop
            prompt = None
