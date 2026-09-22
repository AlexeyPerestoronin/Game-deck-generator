import json
import random
import time
import google
import google.genai

from classproperties import classproperty

from . import tools, i_agent
from .. import logger

__all__ = [
    'GoogleAIStudioModelsSpecifications',
    'GoogleAI',
]


class RateLimiter:
    """Simple rate limiter to respect per-minute request limits of the model."""

    def __init__(self, max_calls_per_min: int):
        self.__min_interval = 60 / max_calls_per_min
        self.__last_call_time = 0.0

    def wait_if_needed(self):
        current_time = time.perf_counter()
        time_since_last_call = current_time - self.__last_call_time
        if time_since_last_call < self.__min_interval:
            sleep_time = self.__min_interval - time_since_last_call
            time.sleep(sleep_time + 1)
        self.__last_call_time = time.perf_counter()


class UsageStats:
    """Accumulates token usage and cost across agent requests."""

    def __init__(self):
        self.requests = 0
        self.input_tokens = 0
        self.output_tokens = 0
        self.cached_tokens = 0
        self.total_cost_usd = 0.0


class GoogleAIStudioModelsSpecifications:
    """Holds model name and rate/token limits for a specific Gemini model from Google AI Studio."""

    @classmethod
    def from_str(cls, model: str) -> 'GoogleAIStudioModelsSpecifications':
        if model == "Gemini-3.8-Flash":
            return GoogleAIStudioModelsSpecifications("gemini-3.8-flash", 5, 250000, 20, 500000)
        elif model == "Gemini-3.1-Flash-Lite":
            return GoogleAIStudioModelsSpecifications("gemini-3.1-flash-lite", 15, 250000, 500, 500000)
        else:
            raise Exception("unexpected model google ai model specification")

    def __init__(self, model: str, rpm: int, tpm: int, rpd: int, tls: int):
        self.__model = model  # google ai studio model name
        self.__rpm = rpm  # request per minute
        self.__tpm = tpm  # tokens per minute
        self.__rpd = rpd  # request per day
        self.__tls = tls  # tokens limit per session

    @property
    def model(self) -> str:
        return self.__model

    @property
    def rpm(self) -> int:
        return self.__rpm

    @property
    def tpm(self) -> int:
        return self.__tpm

    @property
    def rpd(self) -> int:
        return self.__rpd

    @property
    def tls(self) -> int:
        return self.__tls


class GoogleAI(i_agent.IAgent):
    """Gemini agent implementation using Google GenAI SDK with tool calling support."""

    def __init__(self, tools: tools.ITools, logger: logger.ILogger, model_specification: GoogleAIStudioModelsSpecifications):
        self.__tools = tools
        self.__logger = logger
        self.__model_specification = model_specification
        self.__rate_limiter = RateLimiter(self.__model_specification.rpm)

        self._message = None
        with open('API_KEY_GEMINI', encoding='utf-8') as f:
            api_key = f.read().strip()
        self.__client = google.genai.Client(api_key=api_key)

        # Чат должен жить весь цикл AgentLoop (история + implicit caching).
        self.__chat = self.__client.chats.create(
            model=self.__model_specification.model,
            config=google.genai.types.GenerateContentConfig(
                tools=self.__gemini_tools(self.__tools.list),
                automatic_function_calling=google.genai.types.AutomaticFunctionCallingConfig(disable=True),
            ),
        )
        self.__usage_stats = UsageStats()

    # i_agent.IAgent
    @classproperty
    def name(cls) -> str:
        return 'GoogleAI'

    # i_agent.IAgent
    @property
    def tokens_limit(self) -> int:
        return self.__model_specification.tls

    # i_agent.IAgent
    @property
    def consumed_tokens(self) -> int:
        return int(self.__usage_stats.input_tokens) + int(self.__usage_stats.output_tokens)

    # i_agent.IAgent
    def iteration(self, prompt: str) -> bool:
        if prompt:
            self.__logger\
                .log_line("user prompt:")\
                .log_line('```')\
                .log_line(f'{prompt}')\
                .log_line('```')
            self._message = prompt

        response = self.__request(self._message)
        self._message = None

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
            fc_results = []
            for i, fc in enumerate(function_calls, 1):
                arg_str = getattr(fc, "args", None)
                self.__logger.log_line(f"{i}. {fc.name}({arg_str})")
                try:
                    args = self.__fc_args(arg_str)
                    result = self.__tools.call(fc.name, **args)
                    self.__logger.log_str(f" → success")\
                        .log_line("```")\
                        .log_line(f"read/write {len(result)} symbols" if fc.name in ("read_file", "write_file") else result)\
                        .log_line("```")
                except Exception as e:
                    result = f"execution error: {e}"
                    self.__logger.log_str(f" → fail → {result}")

                fc_results.append(google.genai.types.Part.from_function_response(
                    name=fc.name,
                    response={"result": result},
                ))

            self._message = fc_results
            return False
        return True

    # i_agent.IAgent
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
        for attempt in range(1, 24):
            try:
                self.__rate_limiter.wait_if_needed()
                self.__logger.log_line(f"request ademption №{attempt} → ")
                response = self.__chat.send_message(message)
                self.__logger.log_str("success!")
                break
            except Exception as e:
                code = e.details["error"]["code"]
                if code == 429:
                    self.__logger.log_str(f"fail 429: model limit exceeded!")
                elif code == 503:
                    delay = random.randint(5, 20)
                    self.__logger.log_str(f"fail 503: RPM exceeded → waiting {delay}s ... {e}")
                    time.sleep(delay)
                else:
                    self.__logger.log_str(f"❗unexpected exception: {e}")

        self.__usage_stats.requests += 1
        usage = getattr(response, "usage_metadata", None)
        if usage:
            self.__usage_stats.input_tokens += getattr(usage, "prompt_token_count", 0) or 0
            self.__usage_stats.output_tokens += getattr(usage, "candidates_token_count", 0) or 0
            self.__usage_stats.cached_tokens += getattr(usage, "cached_content_token_count", 0) or 0
        return response

    def __gemini_tools(self, xai_tools) -> list:
        # Convert Tool descriptors (or wrapped xai tools) to Gemini FunctionDeclarations.
        declarations = []
        empty_schema = {"type": "object", "properties": {}}
        for tool_obj in xai_tools:
            if hasattr(tool_obj, "name") and hasattr(tool_obj, "description") and hasattr(tool_obj, "parameters"):
                # direct Tool (from DefaultTools)
                name = tool_obj.name
                description = tool_obj.description or ""
                parameters = tool_obj.parameters
            else:
                fn = getattr(tool_obj, "function", tool_obj)
                name = getattr(fn, "name", None)
                description = getattr(fn, "description", "") or ""
                parameters = getattr(fn, "parameters", None)
            if isinstance(parameters, str):
                parameters = json.loads(parameters) if parameters else empty_schema
            if not isinstance(parameters, dict):
                parameters = empty_schema
            try:
                decl = google.genai.types.FunctionDeclaration(
                    name=name,
                    description=description,
                    parameters_json_schema=parameters,
                )
            except TypeError:
                decl = google.genai.types.FunctionDeclaration(
                    name=name,
                    description=description,
                    parameters=parameters,
                )
            declarations.append(decl)
        return [google.genai.types.Tool(function_declarations=declarations)]

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
