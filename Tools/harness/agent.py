import uuid
import json
import xai_sdk
import datetime

from typing import Protocol
from . import tools, logger


class UsageStats:
    """Accumulates token usage and cost across agent requests."""

    def __init__(self):
        self.requests = 0
        self.input_tokens = 0
        self.output_tokens = 0
        self.cached_tokens = 0
        self.total_cost_usd = 0.0


class Agent(Protocol):
    """Protocol for pluggable AI agents in the harness."""

    @property
    def name(self) -> str:
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


class Grok(Agent):
    """Grok agent implementation using xAI SDK with tool calling support."""

    def __init__(self, prompt: str, tools: tools.DefaultToolsSet, logger: logger.Logger, token_limit: int = 128000):
        self.__prompt = prompt
        self.__token_limit = token_limit
        self.__conv_id = str(uuid.uuid4())
        self.__logger = logger
        self.__tools = tools
        with open('API_KEY_XAI', encoding='utf-8') as f:
            api_key = f.read().strip()
        self.__client = xai_sdk.Client(
            api_key=api_key,
            metadata=(("x-grok-conv-id", self.__conv_id), ),
        )
        self.__chat_id = str(uuid.uuid4())
        self.__chat = self.__client.chat.create(model="grok-4.6", conversation_id=self.__chat_id, tools=self.__tools.list)
        self.__usage_stats = UsageStats()

    @property
    def name(self) -> str:
        return 'Grok-4.6'

    @property
    def tokens_limit(self) -> int:
        return self.__token_limit

    @property
    def conversation_id(self) -> str:
        return self.__conv_id

    @property
    def chat_id(self) -> str:
        return self.__chat_id

    @property
    def consumed_tokens(self) -> int:
        return int(self.__usage_stats.input_tokens) + int(self.__usage_stats.output_tokens)

    @property
    def consumed_usd(self) -> float:
        return float(self.__usage_stats.total_cost_usd)

    def iteration(self) -> bool:
        """
        Выполняет один шаг агента: отправляет накопленный контекст (или первый промпт),
        получает ответ модели и обрабатывает tool calls при их наличии.

        Логика завершения цикла:
        - Возвращает False, если модель запросила инструменты (tool_calls).
          После выполнения инструментов и append tool_result внешний цикл
          вызовет iteration() ещё раз, чтобы модель могла продолжить.
        - Возвращает True, если модель дала обычный текстовый ответ без tool_calls.
          Это сигнал, что агент считает задачу выполненной → AgentLoop завершает работу.

        Важно: сразу после sample() мы append'им ответ ассистента в историю чата.
        Без этого история диалога (user → assistant(tool request) → tool) будет неполной,
        и модель может не суметь корректно завершить цикл или потеряет контекст.
        """
        if self.__prompt:
            self.__logger\
                .log_line("user prompt:")\
                .log_line('```')\
                .log_line(f'{self.__prompt}')\
                .log_line('```')
            self.__chat.append(xai_sdk.chat.user(self.__prompt))
            self.__prompt = None

        response = self.__request()

        # КРИТИЧЕСКИ ВАЖНО для корректной истории и логики завершения:
        # Сохраняем ответ модели (assistant turn) до обработки tool results.
        self.__chat.append(response)

        if getattr(response, 'content', None):
            self.__logger\
                .log_line("agent content:")\
                .log_line('```')\
                .log_line(f'{response.content}')\
                .log_line('```')

        if response.tool_calls:
            self.__logger.log_line("agent request tools:")
            for i, tool_call in enumerate(response.tool_calls, 1):
                raw_args = getattr(tool_call.function, 'arguments', '') or '{}'
                try:
                    args = json.loads(raw_args)
                    result = self.__tools.call(tool_call.function.name, **args)
                    status = 'success'
                except Exception as e:
                    result = f"execution error: {e}"
                    status = 'fail'
                    args = raw_args
                self.__chat.append(xai_sdk.chat.tool_result(result))
                arg_str = json.dumps(args, ensure_ascii=False) if isinstance(args, (dict, list)) else str(args)
                self.__logger.log_line(f"{i}. {tool_call.function.name}({arg_str}) → {status}")
            return False

        # Модель ответила без вызовов инструментов → считаем это финальным ответом.
        return True

    def finish(self):
        self.__logger\
            .log_line("Sessions statistic")\
            .log_line(f"- total requests: {self.__usage_stats.requests}")\
            .log_line(f"- total tokens: {self.consumed_tokens}")\
            .log_line(f"    - input tokens: {self.__usage_stats.cached_tokens + self.__usage_stats.input_tokens}")\
            .log_line(f"        - cached tokens: {self.__usage_stats.cached_tokens}")\
            .log_line(f"    - output tokens: {self.__usage_stats.output_tokens}")\
            .log_line(f"- total cost: {self.__usage_stats.total_cost_usd}$")

    def __request(self):
        response = self.__chat.sample()
        self.__usage_stats.requests += 1
        if response.usage:
            self.__usage_stats.input_tokens += response.usage.prompt_tokens or 0
            self.__usage_stats.output_tokens += response.usage.completion_tokens or 0
            details = getattr(response.usage, "prompt_tokens_details", None)
            if details:
                self.__usage_stats.cached_tokens += getattr(details, "cached_tokens", 0) or 0
        if hasattr(response, "cost_usd") and response.cost_usd is not None:
            self.__usage_stats.total_cost_usd += response.cost_usd
        return response


class AgentLoop:
    """Drives the agent through iterations, enforcing token and iteration limits."""

    def __init__(self, iteration_limit: int, agent: Agent, logger: logger.Logger):
        self.__iteration_limit = iteration_limit
        self.__iteration = 0
        self.__agent = agent
        self.__logger = logger

    def check_session_token_limit(self) -> bool:
        """Return True if within limit (or user approved over limit)."""
        limit = self.__agent.tokens_limit
        consumed = self.__agent.consumed_tokens
        if consumed > limit:
            user_decision = input(f"consumed tokens have exceeded the limit ({consumed} > {limit}) → resume execution? [y/N]: ").strip().lower()
            return user_decision in ("y", "yes")
        return True

    def check_session_iteration_limit(self) -> bool:
        """Return True if within limit (or user approved over limit)."""
        limit = self.__iteration_limit
        consumed = self.__iteration
        if consumed > limit:
            user_decision = input(f"loop iteration quantity has exceeded the limit ({consumed} > {limit}) → resume execution? [y/N]: ").strip().lower()
            return user_decision in ("y", "yes")
        return True

    def start_loop(self):
        agent_name = f"{self.__agent.name}-AI-agent"
        self.__logger\
            .log_line(f"# Agent-loop session:")\
            .log_line(f"- agent: {agent_name}")\
            .log_line(f"- iteration limit: {self.__iteration_limit}")\
            .log_line(f"- tokens limit: {self.__agent.tokens_limit}")\
            .log_line(f"- start time: {datetime.datetime.now().strftime('%Y-%m-%d %H:%M')}")\
            .log_line()\
            .log_line("# Iterations:")\
            .log_line()

        while True:
            self.__iteration += 1
            self.__logger.log_line(f"## Iteration №{self.__iteration}:")

            if not self.check_session_token_limit():
                message = "tokens limit exceed"
                self.__logger.log_line(f"⚠️: {message}")
                raise Exception(message)

            if not self.check_session_iteration_limit():
                message = "iteration limit exceed"
                self.__logger.log_line(f"⚠️: {message}")
                raise Exception(message)

            if self.__agent.iteration():
                self.__logger.log_line(f"# Session results:")
                self.__agent.finish()
                break
