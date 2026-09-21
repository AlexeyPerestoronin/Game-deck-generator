import uuid
import json
import xai_sdk

from classproperties import classproperty

from . import tools, i_agent
from .. import logger

__all__ = [
    'Grok',
]


class UsageStats:
    """Accumulates token usage and cost across agent requests."""

    def __init__(self):
        self.requests = 0
        self.input_tokens = 0
        self.output_tokens = 0
        self.cached_tokens = 0
        self.total_cost_usd = 0.0


class Grok(i_agent.IAgent):
    """Grok agent implementation using xAI SDK with tool calling support."""

    def __init__(self, prompt: str, tools: tools.ITools, logger: logger.ILogger, token_limit: int = 128000):
        self._prompt = prompt
        self._token_limit = token_limit
        self._conv_id = str(uuid.uuid4())
        self._logger = logger
        self._tools = tools
        # ---
        with open('API_KEY_XAI', encoding='utf-8') as f:
            api_key = f.read().strip()
        # ---
        self._client = xai_sdk.Client(
            api_key=api_key,
            metadata=(("x-grok-conv-id", self._conv_id), ),
        )
        self.__chat_id = str(uuid.uuid4())
        self.__chat = self._client.chat.create(model="grok-4.6", conversation_id=self.__chat_id, tools=self.__grok_tools(self._tools.list))
        self.__usage_stats = UsageStats()

    @classproperty
    def name(cls) -> str:
        return 'Grok-4.6'

    @property
    def tokens_limit(self) -> int:
        return self._token_limit

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

    def iteration(self) -> bool:
        # Caching for cost saving (USD) is supported implicitly by keeping the same chat instance
        # (with conversation_id) across the whole AgentLoop, analogous to GoogleAI.
        if self._prompt:
            self._logger\
                .log_line("user prompt:")\
                .log_line('```')\
                .log_line(f'{self._prompt}')\
                .log_line('```')
            self.__chat.append(xai_sdk.chat.user(self._prompt))
            self._prompt = None

        response = self.__request()

        # Сохраняем ответ ассистента (содержит tool_calls с их уникальными ID)
        self.__chat.append(response)

        if getattr(response, 'content', None):
            self._logger\
                .log_line("agent content:")\
                .log_line('```')\
                .log_line(f'{response.content}')\
                .log_line('```')

        if response.tool_calls:
            self._logger.log_line("agent request tools:")

            # Собираем все результаты инструментов параллельно, чтобы отправить их корректно
            for i, tool_call in enumerate(response.tool_calls, 1):
                raw_args = getattr(tool_call.function, 'arguments', '') or '{}'
                tool_call_id = tool_call.id  # ФИКС: Обязательно вытаскиваем ID вызова инструмента

                try:
                    args = json.loads(raw_args)
                    result = self._tools.call(tool_call.function.name, **args)
                    status = "success"
                except Exception as e:
                    result = f"execution error: {e}"
                    status = f"fail: {e}"
                    args = raw_args

                # ФИКС: Передаем tool_call_id, чтобы xAI API понимал, к какому вызову относится этот результат
                self.__chat.append(xai_sdk.chat.tool_result(result, tool_call_id=tool_call_id))

                arg_str = json.dumps(args, ensure_ascii=False) if isinstance(args, (dict, list)) else str(args)
                self._logger.log_line(f"{i}. {tool_call.function.name}({arg_str}) → {status}")

            # Возвращаем False: цикл должен продолжиться, так как мы только что дали модели данные из файлов/git
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
        response = self.__chat.sample()
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
