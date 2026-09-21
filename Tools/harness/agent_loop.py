import os
import datetime

from . import agents, logger

__all__ = [
    'AgentLoop',
]

class AgentLoop:
    """Drives the agent through iterations, enforcing token and iteration limits."""

    def __init__(self, safe_mode: bool, logger: logger.ILogger, settings):
        self.__safe_mode = safe_mode
        self.__logger = logger
        self.__settings = settings

        self.__iteration = 0
        self.__iteration_limit = self.__settings["iteration-limit"]
        token_limit = self.__settings["token-limit"]

        requested_tools = self.__settings["tools"]
        if requested_tools == agents.tools.DefaultTools.name:
            self.__tools = agents.tools.DefaultTools(self.__safe_mode)
        elif requested_tools == agents.tools.DefaultTools.name:
            self.__tools = agents.tools.FSTools(cwd=os.getcwd(), allowed_dirs=self.__settings["allowed-dirs"])
        else:
            raise Exception("unexpected type of agent tools")

        prompt = self.__settings["prompt"]
        requested_model = self.__settings["model"]
        if requested_model == agents.Grok.name:
            self.__agent = agents.Grok(prompt, self.__tools, self.__logger, token_limit)
        elif requested_model == agents.Gemini.name:
            self.__agent = agents.Gemini(prompt, self.__tools, self.__logger, token_limit)
        else:
            raise Exception("unexpected model of agent")

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
