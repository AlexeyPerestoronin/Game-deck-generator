import json
import random
import time
import pathlib
import google.genai
from classproperties import classproperty
from . import tools, i_agent
from .. import logger

__all__ = [
    'GoogleModels',
    'GoogleAI',
]


class RateLimiter:
    """Simple rate limiter."""

    def __init__(self, max_calls_per_min: int):
        self.__min_interval = 60 / max_calls_per_min
        self.__last_call_time = 0.0

    def wait_if_needed(self):
        current_time = time.perf_counter()
        time_since_last_call = current_time - self.__last_call_time
        if time_since_last_call < self.__min_interval:
            time.sleep(self.__min_interval - time_since_last_call + 1)
        self.__last_call_time = time.perf_counter()


class UsageStats:
    """Accumulates usage."""

    def __init__(self):
        self.requests = 0
        self.input_tokens = 0
        self.output_tokens = 0
        self.cached_tokens = 0
        self.total_cost_usd = 0.0


class GoogleModels:
    """Model specifications."""

    @classmethod
    def from_str(cls, model: str) -> 'GoogleModels':
        if model == "Gemini 3.8 Flash":
            return cls("gemini-3.8-flash", 5, 250000, 20, -1)
        elif model == "Gemini 3.5 Flash Lite":
            return cls("gemini-3.5-flash-lite", 15, 250000, 500, -1)
        elif model == "Gemini 3.1 Flash Lite":
            return cls("gemini-3.1-flash-lite", 15, 250000, 500, -1)
        raise ValueError(f"Unknown model: {model}")

    def __init__(self, model: str, rpm: int, tpm: int, rpd: int, tls: int):
        self.model = model
        self.rpm = rpm
        self.tpm = tpm
        self.rpd = rpd
        self.tls = tls


class GoogleAI(i_agent.IAgent):
    """Gemini agent."""

    def __init__(self, tools_handler: tools.ITools, log: logger.ILogger, spec: GoogleModels):
        self.__tools = tools_handler
        self.__logger = log
        self.__spec = spec
        self.__rate_limiter = RateLimiter(spec.rpm)
        self._message = None
        self.__usage_stats = UsageStats()

        with open('API_KEY_GEMINI', encoding='utf-8') as f:
            api_key = f.read().strip()
        self.__client = google.genai.Client(api_key=api_key)

        self.__chat = self.__client.chats.create(
            model=spec.model,
            config=google.genai.types.GenerateContentConfig(
                tools=self.__prepare_gemini_tools(self.__tools.list),
                automatic_function_calling=google.genai.types.AutomaticFunctionCallingConfig(disable=True),
            ),
        )

    # i_agent.IAgent
    @classproperty
    def vendor(cls) -> str:
        return 'GoogleAI'

    # i_agent.IAgent
    @property
    def model(self) -> str:
        self.__spec.model

    @property
    def tokens_limit(self) -> int:
        return self.__spec.tls

    @property
    def consumed_tokens(self) -> int:
        return self.__usage_stats.input_tokens + self.__usage_stats.output_tokens

    # i_agent.IAgent
    def iteration(self, prompt: str = None) -> bool:
        if prompt:
            self.__log_prompt(prompt)
            self._message = prompt

        response = self.__execute_request(self._message)
        self._message = None

        text = self.__get_response_text(response)
        if text:
            self.__log_agent_content(text)

        fcs = self.__get_function_calls(response)
        if fcs:
            self._message = self.__handle_tool_calls(fcs)
            return False
        return True

    def finish(self):
        self.__logger.log_line("Sessions statistic")\
            .log_line(f"- total requests: {self.__usage_stats.requests}")\
            .log_line(f"- total tokens: {self.consumed_tokens}")\
            .log_line(f"    - input: {self.__usage_stats.input_tokens} (cached: {self.__usage_stats.cached_tokens})")\
            .log_line(f"    - output: {self.__usage_stats.output_tokens}")

    # i_agent.IAgent
    def dump_session(self, dump_file: pathlib.Path):
        history = [content.model_dump(mode="json") for content in self.__chat.get_history(raw=True)]
        pending_message = self.__pending_message_to_dict()
        data = {
            "usage_stats": self.__usage_stats_to_dict(),
            "history": history,
            "pending_message": pending_message,
        }
        with open(dump_file, "w", encoding="utf-8") as file:
            json.dump(data, file, ensure_ascii=False, indent=2)

    # i_agent.IAgent
    def reload_session(self, dump_file: pathlib.Path):
        with open(dump_file, "r", encoding="utf-8") as file:
            data = json.load(file)

        self.__usage_stats_from_dict(data.get("usage_stats", {}))
        history = [google.genai.types.Content.model_validate(item) for item in data.get("history", [])]
        self.__chat = self.__client.chats.create(
            model=self.__spec.model,
            history=history,
            config=google.genai.types.GenerateContentConfig(
                tools=self.__prepare_gemini_tools(self.__tools.list),
                automatic_function_calling=google.genai.types.AutomaticFunctionCallingConfig(disable=True),
            ),
        )
        self._message = self.__pending_message_from_dict(data.get("pending_message"))

    def __usage_stats_to_dict(self) -> dict:
        # snapshot of counters stored alongside chat history
        return {
            "requests": self.__usage_stats.requests,
            "input_tokens": self.__usage_stats.input_tokens,
            "output_tokens": self.__usage_stats.output_tokens,
            "cached_tokens": self.__usage_stats.cached_tokens,
            "total_cost_usd": self.__usage_stats.total_cost_usd,
        }

    def __usage_stats_from_dict(self, stats: dict):
        # restore counters without replacing the UsageStats instance
        self.__usage_stats.requests = stats.get("requests", 0)
        self.__usage_stats.input_tokens = stats.get("input_tokens", 0)
        self.__usage_stats.output_tokens = stats.get("output_tokens", 0)
        self.__usage_stats.cached_tokens = stats.get("cached_tokens", 0)
        self.__usage_stats.total_cost_usd = stats.get("total_cost_usd", 0.0)

    def __pending_message_to_dict(self) -> dict | None:
        # _message is either a user prompt, function-response parts, or empty
        if isinstance(self._message, str):
            return {"type": "str", "value": self._message}
        if isinstance(self._message, list):
            return {
                "type": "parts",
                "value": [part.model_dump(mode="json") for part in self._message],
            }
        return None

    def __pending_message_from_dict(self, pending_message: dict | None):
        if pending_message is None:
            return None
        if pending_message.get("type") == "str":
            return pending_message.get("value")
        return [google.genai.types.Part.model_validate(part) for part in pending_message.get("value", [])]

    def __log_prompt(self, prompt: str):
        self.__logger.log_line("user prompt:").log_line('```').log_line(prompt).log_line('```')

    def __log_agent_content(self, text: str):
        self.__logger.log_line("agent content:").log_line('```').log_line(text).log_line('```')

    def __execute_request(self, message):
        for attempt in range(1, 24):
            try:
                self.__rate_limiter.wait_if_needed()
                response = self.__chat.send_message(message)
                self.__update_stats(response)
                return response
            except Exception as e:
                code = getattr(e, "code", None)
                if code == 429:
                    self.__logger.log_str(" fail 429: model limit exceeded!")
                elif code == 503:
                    delay = random.randint(5, 20)
                    self.__logger.log_str(f" fail 503: waiting {delay}s...")
                    time.sleep(delay)
                else:
                    self.__logger.log_str(f"unexpected exception: {e}")
                    raise
        raise Exception("Request failed after max retries")

    def __update_stats(self, response):
        self.__usage_stats.requests += 1
        usage = getattr(response, "usage_metadata", None)
        if usage:
            self.__usage_stats.input_tokens += (getattr(usage, "prompt_token_count", 0) or 0)
            self.__usage_stats.output_tokens += (getattr(usage, "candidates_token_count", 0) or 0)
            self.__usage_stats.cached_tokens += (getattr(usage, "cached_content_token_count", 0) or 0)

    def __prepare_gemini_tools(self, xai_tools) -> list:
        declarations = []
        for t in xai_tools:
            parameters = t.parameters if isinstance(t.parameters, dict) else {}
            declarations.append(google.genai.types.FunctionDeclaration(
                name=t.name,
                description=t.description or "",
                parameters_json_schema=parameters,
            ))
        return [google.genai.types.Tool(function_declarations=declarations)]

    def __get_response_text(self, response) -> str | None:
        try:
            return response.text
        except Exception:
            return None

    def __get_function_calls(self, response) -> list:
        fcs = []
        candidates = getattr(response, "candidates", []) or []
        for candidate in candidates:
            parts = getattr(candidate.content, "parts", []) or []
            for part in parts:
                if part.function_call:
                    fcs.append(part.function_call)
        return [fc for fc in fcs if fc.name]

    def __handle_tool_calls(self, fcs) -> list:
        self.__logger.log_line("agent request tools:")
        results = []
        for i, fc in enumerate(fcs, 1):
            args = dict(fc.args) if fc.args else {}

            log_arg = "..." if fc.name == 'write_file' else args
            self.__logger.log_line(f"{i}. {fc.name}({log_arg})")
            try:
                result = self.__tools.call(fc.name, **args)
                self.__logger.log_str(" → success")
                if fc.name in ['read_file', 'write_file']:
                    self.__logger.log_str(f" → read/write {len(result)}symbols")
                else:
                    self.__logger.log_line("```").log_line(result).log_line("```")
            except Exception as e:
                result = f"execution error: {e}"
                self.__logger.log_str(f" → fail → {result}")
            results.append(google.genai.types.Part.from_function_response(name=fc.name, response={"result": result}))
        return results
