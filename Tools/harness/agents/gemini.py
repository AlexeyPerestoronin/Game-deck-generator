import uuid
import json

from classproperties import classproperty
from google import genai
from google.genai import types

from . import tools, i_agent
from .. import logger

__all__ = [
    'Gemini',
]


class UsageStats:
    """Accumulates token usage and cost across agent requests."""

    def __init__(self):
        self.requests = 0
        self.input_tokens = 0
        self.output_tokens = 0
        self.cached_tokens = 0
        self.total_cost_usd = 0.0


class Gemini(i_agent.IAgent):
    """Gemini agent implementation using Google GenAI SDK with tool calling support."""

    def __init__(self, prompt: str, tools: tools.ITools, logger: logger.ILogger, token_limit: int = 128000):
        self.__prompt = prompt
        self.__token_limit = token_limit
        self.__conv_id = str(uuid.uuid4())
        self.__logger = logger
        self.__tools = tools
        self.__types = types
        self.__pending_tool_parts = None
        with open('API_KEY_GEMINI', encoding='utf-8') as f:
            api_key = f.read().strip()
        self.__client = genai.Client(api_key=api_key)
        self.__chat_id = str(uuid.uuid4())

        # Чат должен жить весь цикл AgentLoop (история + implicit caching).
        self.__chat = self.__client.chats.create(
            model="gemini-3.8-flash",
            config=types.GenerateContentConfig(
                tools=self.__gemini_tools(self.__tools.list),
                automatic_function_calling=types.AutomaticFunctionCallingConfig(disable=True),
            ),
        )
        self.__usage_stats = UsageStats()

    @classproperty
    def name(cls) -> str:
        return 'Gemini-3.8-Flash'

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
        if self.__prompt:
            self.__logger\
                .log_line("user prompt:")\
                .log_line('```')\
                .log_line(f'{self.__prompt}')\
                .log_line('```')
            message = self.__prompt
            self.__prompt = None
        else:
            message = self.__pending_tool_parts
            self.__pending_tool_parts = None

        response = self.__request(message)

        text = self.__response_text(response)
        if text:
            self.__logger\
                .log_line("agent content:")\
                .log_line('```')\
                .log_line(f'{text}')\
                .log_line('```')

        function_calls = self.__response_function_calls(response)
        if function_calls:
            self.__logger.log_line("agent request tools:")
            pending = []
            for i, fc in enumerate(function_calls, 1):
                raw_args = getattr(fc, "args", None)
                try:
                    args = self.__fc_args(raw_args)
                    result = self.__tools.call(fc.name, **args)
                    status = "success"
                except Exception as e:
                    result = f"execution error: {e}"
                    status = f"fail: {e}"
                    args = raw_args

                pending.append(
                    self.__types.Part.from_function_response(
                        name=fc.name,
                        response={"result": result},
                    )
                )

                arg_str = json.dumps(args, ensure_ascii=False) if isinstance(args, (dict, list)) else str(args)
                self.__logger.log_line(f"{i}. {fc.name}({arg_str}) → {status}")

            self.__pending_tool_parts = pending
            return False

        return True

    def finish(self):
        self.__logger\
            .log_line("Sessions statistic")\
            .log_line(f"- total requests: {self.__usage_stats.requests}")\
            .log_line(f"- total tokens: {self.consumed_tokens}")\
            .log_line(f"    - input tokens: {self.__usage_stats.input_tokens} (from stats)")\
            .log_line(f"        - cached tokens: {self.__usage_stats.cached_tokens}")\
            .log_line(f"    - output tokens: {self.__usage_stats.output_tokens}")\
            .log_line(f"- total cost: {self.__usage_stats.total_cost_usd}$")

    def __request(self, message):
        response = self.__chat.send_message(message)
        self.__usage_stats.requests += 1
        usage = getattr(response, "usage_metadata", None)
        if usage:
            self.__usage_stats.input_tokens += getattr(usage, "prompt_token_count", 0) or 0
            self.__usage_stats.output_tokens += getattr(usage, "candidates_token_count", 0) or 0
            self.__usage_stats.cached_tokens += getattr(usage, "cached_content_token_count", 0) or 0
        return response

    def __gemini_tools(self, xai_tools) -> list:
        types = self.__types
        declarations = []
        empty_schema = {"type": "object", "properties": {}}
        for tool_obj in xai_tools:
            fn = getattr(tool_obj, "function", tool_obj)
            name = getattr(fn, "name", None)
            description = getattr(fn, "description", "") or ""
            parameters = getattr(fn, "parameters", None)
            if isinstance(parameters, str):
                parameters = json.loads(parameters) if parameters else empty_schema
            if not isinstance(parameters, dict):
                parameters = empty_schema
            try:
                decl = types.FunctionDeclaration(
                    name=name,
                    description=description,
                    parameters_json_schema=parameters,
                )
            except TypeError:
                decl = types.FunctionDeclaration(
                    name=name,
                    description=description,
                    parameters=parameters,
                )
            declarations.append(decl)
        return [types.Tool(function_declarations=declarations)]

    def __response_text(self, response) -> str | None:
        try:
            text = response.text
        except Exception:
            text = None
        return text or None

    def __response_function_calls(self, response) -> list:
        fcs = getattr(response, "function_calls", None)
        if fcs:
            return [fc for fc in fcs if getattr(fc, "name", None)]
        result = []
        for candidate in getattr(response, "candidates", None) or []:
            content = getattr(candidate, "content", None)
            for part in getattr(content, "parts", None) or []:
                fc = getattr(part, "function_call", None)
                if fc and getattr(fc, "name", None):
                    result.append(fc)
        return result

    def __fc_args(self, raw_args) -> dict:
        if not raw_args:
            return {}
        if isinstance(raw_args, dict):
            return raw_args
        return dict(raw_args)
