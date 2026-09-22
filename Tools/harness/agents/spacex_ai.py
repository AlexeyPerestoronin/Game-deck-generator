import json
import uuid
import xai_sdk
from classproperties import classproperty
from . import tools, i_agent
from .. import logger

__all__ = [
    'SpaceXModels',
    'SpaceXAI',
]


class UsageStats:
    """Accumulates usage."""

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
    """Grok agent."""

    def __init__(self, tools_handler: tools.ITools, log: logger.ILogger, spec: SpaceXModels):
        self.__tools = tools_handler
        self.__logger = log
        self.__spec = spec
        self.__conv_id = str(uuid.uuid4())
        self.__chat_id = str(uuid.uuid4())
        self.__usage_stats = UsageStats()

        with open('API_KEY_XAI', encoding='utf-8') as f:
            api_key = f.read().strip()
        self.__client = xai_sdk.Client(
            api_key=api_key,
            metadata=(("x-grok-conv-id", self.__conv_id), ),
        )
        self.__chat = self.__client.chat.create(
            model="grok-4.6",
            conversation_id=self.__chat_id,
            tools=self.__prepare_grok_tools(self.__tools.list),
        )

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

    # i_agent.IAgent
    def iteration(self, prompt: str) -> bool:
        if prompt:
            self.__log_prompt(prompt)
            self.__chat.append(xai_sdk.chat.user(prompt))

        response = self.__execute_request()
        self.__chat.append(response)

        text = self.__get_response_text(response)
        if text:
            self.__log_agent_content(text)

        if response.tool_calls:
            self.__handle_tool_calls(response.tool_calls)
            return False
        return True

    def finish(self):
        self.__logger.log_line("Sessions statistic")\
            .log_line(f"- total requests: {self.__usage_stats.requests}")\
            .log_line(f"- total tokens: {self.consumed_tokens}")\
            .log_line(f"    - input tokens: {self.__usage_stats.input_tokens} (from stats)")\
            .log_line(f"        - cached tokens: {self.__usage_stats.cached_tokens}")\
            .log_line(f"    - output tokens: {self.__usage_stats.output_tokens}")\
            .log_line(f"- total cost: {self.__usage_stats.total_cost_usd}$")

    def __log_prompt(self, prompt: str):
        self.__logger.log_line("user prompt:").log_line('```').log_line(prompt).log_line('```')

    def __log_agent_content(self, text: str):
        self.__logger.log_line("agent content:").log_line('```').log_line(text).log_line('```')

    def __execute_request(self):
        response = self.__chat.sample()
        self.__update_stats(response)
        return response

    def __update_stats(self, response):
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

    def __prepare_grok_tools(self, xai_tools) -> list:
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

    def __get_response_text(self, response) -> str | None:
        return getattr(response, 'content', None)

    def __handle_tool_calls(self, tool_calls):
        self.__logger.log_line("agent request tools:")
        for i, tool_call in enumerate(tool_calls, 1):
            raw_args = getattr(tool_call.function, 'arguments', '') or '{}'
            tool_call_id = tool_call.id

            try:
                args = json.loads(raw_args)
                result = self.__tools.call(tool_call.function.name, **args)
                status = "success"
            except Exception as e:
                result = f"execution error: {e}"
                status = f"fail: {e}"
                args = raw_args

            tool_result = xai_sdk.chat.tool_result(result, tool_call_id=tool_call_id)
            self.__chat.append(tool_result)

            arg_str = json.dumps(args, ensure_ascii=False) if isinstance(args, (dict, list)) else str(args)
            self.__logger\
                .log_line(f"{i}. {tool_call.function.name}({arg_str}) → {status}")\
                .log_line("```")\
                .log_line(f"{result}")\
                .log_line("```")
