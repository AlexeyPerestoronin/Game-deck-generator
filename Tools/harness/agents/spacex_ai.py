import uuid
import json
import xai_sdk

from classproperties import classproperty

from . import tools, i_agent
from .. import logger

__all__ = [
    'SpaceXModels',
    'SpaceXAI',
]


class UsageStats:
    """Accumulates token usage and cost across agent requests."""

    def __init__(self):
        self.requests = 0
        self.input_tokens = 0
        self.output_tokens = 0
        self.cached_tokens = 0
        self.total_cost_usd = 0.0


class SpaceXModels:
    """Model specifications."""

    @classmethod
    def from_str(cls, model: str) -> 'SpaceXModels':
        if model == "Grok 4.6":
            return cls("grok-4.6", 125000)
        raise ValueError(f"Unknown model: {model}")

    def __init__(self, model: str, tls: int):
        self.model = model
        self.tls = tls


class SpaceXAI(i_agent.IAgent):
    """Grok agent implementation using xAI SDK with tool calling support."""

    def __init__(self, tools: tools.ITools, logger: logger.ILogger, spec: SpaceXModels):
        self._logger = logger
        self._tools = tools
        self.__spec = spec

        self._conv_id = str(uuid.uuid4())
        # ---
        with open('API_KEY_XAI', encoding='utf-8') as f:
            api_key = f.read().strip()
        # ---
        self._client = xai_sdk.Client(
            api_key=api_key,
            metadata=(("x-grok-conv-id", self._conv_id), ),
        )
        self.__chat_id = str(uuid.uuid4())
        self._chat = self._client.chat.create(model="grok-4.6", conversation_id=self.__chat_id, tools=self.__grok_tools(self._tools.list))
        self.__usage_stats = UsageStats()

    # i_agent.IAgent
    @classproperty
    def vendor(cls) -> str:
        return 'SpaceXAI'

    # i_agent.IAgent
    @property
    def model(self) -> str:
        self.__spec.model

    @property
    def tokens_limit(self) -> int:
        return self.__spec.tls

    @property
    def conversation_id(self) -> str:
        return self._conv_id

    @property
    def chat_id(self) -> str:
        return self.__chat_id

    @property
    def consumed_tokens(self) -> int:
        return int(self.__usage_stats.input_tokens) + int(self.__usage_stats.output_tokens)

    @property
    def consumed_usd(self) -> float:
        return float(self.__usage_stats.total_cost_usd)

    # i_agent.IAgent
    def iteration(self, prompt: str) -> bool:
        if prompt:
            self._logger\
                .log_line("user prompt:")\
                .log_line('```')\
                .log_line(f'{prompt}')\
                .log_line('```')
            self._chat.append(xai_sdk.chat.user(prompt))

        response = self.__request()
        self._chat.append(response)

        if getattr(response, 'content', None):
            self._logger\
                .log_line("agent content:")\
                .log_line('```')\
                .log_line(f'{response.content}')\
                .log_line('```')

        if response.tool_calls:
            self._logger.log_line("agent request tools:")
            for i, tool_call in enumerate(response.tool_calls, 1):
                raw_args = getattr(tool_call.function, 'arguments', '') or '{}'
                tool_call_id = tool_call.id

                try:
                    args = json.loads(raw_args)
                    result = self._tools.call(tool_call.function.name, **args)
                    status = "success"
                except Exception as e:
                    result = f"execution error: {e}"
                    status = f"fail: {e}"
                    args = raw_args

                tool_result = xai_sdk.chat.tool_result(result, tool_call_id=tool_call_id)
                self._chat.append(tool_result)

                arg_str = json.dumps(args, ensure_ascii=False) if isinstance(args, (dict, list)) else str(args)
                self._logger\
                    .log_line(f"{i}. {tool_call.function.name}({arg_str}) → {status}")\
                    .log_line("```")\
                    .log_line(f"{result}")\
                    .log_line("```")

            return False

        return True

    def finish(self):
        self._logger\
            .log_line("Sessions statistic")\
            .log_line(f"- total requests: {self.__usage_stats.requests}")\
            .log_line(f"- total tokens: {self.consumed_tokens}")\
            .log_line(f"    - input tokens: {self.__usage_stats.input_tokens} (from stats)")\
            .log_line(f"        - cached tokens: {self.__usage_stats.cached_tokens}")\
            .log_line(f"    - output tokens: {self.__usage_stats.output_tokens}")\
            .log_line(f"- total cost: {self.__usage_stats.total_cost_usd}$")

    def __request(self):
        response = self._chat.sample()
        self.__usage_stats.requests += 1
        if response.usage:
            # Исправлен подсчет токенов: prompt_tokens в API обычно включает в себя cached_tokens.
            # Мы сохраняем "чистые" значения, предоставляемые API.
            self.__usage_stats.input_tokens += response.usage.prompt_tokens or 0
            self.__usage_stats.output_tokens += response.usage.completion_tokens or 0

            details = getattr(response.usage, "prompt_tokens_details", None)
            if details:
                self.__usage_stats.cached_tokens += getattr(details, "cached_tokens", 0) or 0

        if hasattr(response, "cost_usd") and response.cost_usd is not None:
            self.__usage_stats.total_cost_usd += response.cost_usd
        return response

    def __grok_tools(self, xai_tools) -> list:
        # Convert our Tool (or xai tool) descriptors into xai_sdk.chat.tool objects for the Grok chat.
        result = []
        for t in xai_tools:
            if hasattr(t, "name") and hasattr(t, "description") and hasattr(t, "parameters"):
                # plain Tool descriptor (from DefaultTools)
                result.append(xai_sdk.chat.tool(
                    name=t.name,
                    description=t.description,
                    parameters=t.parameters,
                ))
            else:
                # already an xai_sdk tool object (from FSTools etc)
                result.append(t)
        return result
