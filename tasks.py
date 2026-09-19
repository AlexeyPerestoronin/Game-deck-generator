import invoke


import os
import uuid
import subprocess
import xai_sdk


# ─────────────────────────────────────────────
# 3. ИНСТРУМЕНТЫ (без изменений)
# ─────────────────────────────────────────────

def read_file(path: str) -> str:
    if not os.path.exists(path):
        return f"Ошибка: файл {path} не найден."
    if not os.path.abspath(path).startswith(os.getcwd()):
        return "Ошибка: доступ за пределы рабочей директории запрещён."
    with open(path, "r", encoding="utf-8") as f:
        return f.read()

def write_file(path: str, content: str) -> str:
    with open(path, "w", encoding="utf-8") as f:
        f.write(content)
    return f"Файл {path} успешно сохранён."

def run_shell(command: str) -> str:
    confirm = input(f"Выполнить `{command}`? [y/N]: ").strip().lower()
    if confirm not in ("y", "yes"):
        return "Пользователь запретил выполнение команды."
    try:
        result = subprocess.run(
            command, shell=True, capture_output=True, text=True, timeout=30
        )
        output = result.stdout or result.stderr
        return output if output else "(Команда выполнена без вывода)"
    except subprocess.TimeoutExpired:
        return "Ошибка: превышено время ожидания (30 сек)."

tools_list = [
    xai_sdk.chat.tool(
        name="read_local_file",
        description="Прочитать содержимое текстового файла.",
        parameters={
            "type": "object",
            "properties": {"path": {"type": "string"}},
            "required": ["path"],
        },
    ),
    xai_sdk.chat.tool(
        name="write_local_file",
        description="Записать или перезаписать файл.",
        parameters={
            "type": "object",
            "properties": {
                "path": {"type": "string"},
                "content": {"type": "string"},
            },
            "required": ["path", "content"],
        },
    ),
    xai_sdk.chat.tool(
        name="run_terminal_command",
        description="Выполнить shell-команду.",
        parameters={
            "type": "object",
            "properties": {"command": {"type": "string"}},
            "required": ["command"],
        },
    ),
]

tool_handlers = {
    "read_local_file": read_file,
    "write_local_file": write_file,
    "run_terminal_command": run_shell,
}

# ─────────────────────────────────────────────
# 4. СТАТИСТИКА (с учётом кэша)
# ─────────────────────────────────────────────

class UsageStats:
    def __init__(self):
        self.requests = 0
        self.prompt_tokens = 0
        self.completion_tokens = 0
        self.cached_tokens = 0
        self.total_cost_usd = 0.0

    def add_response(self, response):
        self.requests += 1
        if response.usage:
            self.prompt_tokens += response.usage.prompt_tokens or 0
            self.completion_tokens += response.usage.completion_tokens or 0
            details = getattr(response.usage, "prompt_tokens_details", None)
            if details:
                self.cached_tokens += getattr(details, "cached_tokens", 0) or 0
        if hasattr(response, "cost_usd") and response.cost_usd is not None:
            self.total_cost_usd += response.cost_usd

    def report(self):
        print("\n" + "=" * 40)
        print("📊 СТАТИСТИКА СЕССИИ")
        print("=" * 40)
        print(f"Всего запросов: {self.requests}")
        print(f"Входные токены: {self.prompt_tokens:,}")
        print(f"  (из них кэшировано: {self.cached_tokens:,})")
        if self.prompt_tokens > 0:
            hit_rate = self.cached_tokens / self.prompt_tokens * 100
            print(f"  Cache hit rate: {hit_rate:.1f}%")
        print(f"Выходные токены: {self.completion_tokens:,}")
        print(f"💰 Стоимость: ${self.total_cost_usd:.6f}")
        print("=" * 40)

# ─────────────────────────────────────────────
# 5. ЦИКЛ АГЕНТА (с сохранением истории)
# ─────────────────────────────────────────────


@invoke.task()
def agent_loop(ctx, max_turns: int = 10):
    # ─────────────────────────────────────────────
    # 1. ГЕНЕРАЦИЯ СТАБИЛЬНОГО ID СЕССИИ
    # ─────────────────────────────────────────────
    # Этот ID создаётся ОДИН РАЗ при запуске harness.
    # Все запросы в рамках одной сессии используют его,
    # чтобы попадать на один сервер и переиспользовать кэш.
    conv_id = str(uuid.uuid4())
    print(f"[Cache] Conversation ID: {conv_id}")

    # ─────────────────────────────────────────────
    # 2. СОЗДАНИЕ КЛИЕНТА С МЕТАДАННЫМИ ДЛЯ КЭША
    # ─────────────────────────────────────────────
    # metadata=(("x-grok-conv-id", conv_id),) — это ключевой момент.
    # SDK передаёт этот заголовок с каждым gRPC-запросом [citation:3][citation:16].
    api_key=open('API_KEY_XAI').read()
    client = xai_sdk.Client(
        api_key=open('API_KEY_XAI').read(),
        metadata=(("x-grok-conv-id", conv_id),),
    )

    stats = UsageStats()

    print("🤖 Harness готов. Введите задачу или 'exit'.")
    print("💡 Кэширование активно. Не редактируйте историю вручную — только дополняйте.\n")

    # История диалога. ВАЖНО: используем ОДИН объект chat на всю сессию,
    # чтобы SDK накапливал историю внутри себя и не пересобирал её.
    messages = []
    chat = None

    while True:
        user_input =  "Найти файл build.bat в корне репозитория и минимизируй его содержимое, сократив вывод до кратких статусов." #input("Вы: ").strip()
        if user_input.lower() in ("exit", "quit"):
            break

        # Первый запрос: создаём чат и добавляем системный промпт + задачу.
        # Системный промпт кладём В НАЧАЛО — он статичен и будет закэширован [citation:2].
        if chat is None:
            system_prompt = (
                # "Ты — опытный разработчик в локальном репозитории. "
                # "Используй инструменты для чтения и изменения файлов. "
                # "Всегда сначала читай файл, прежде чем его менять. "
                # "Для проверки используй run_terminal_command (pytest, py_compile)."
            )
            result = tool_handlers['run_terminal_command']('ls -la && find . -maxdepth 3 -name "build.bat" -o -name "*.bat" 2>/dev/null | head -50')
            result = xai_sdk.chat.tool_result(result)
            
            
            chat = client.chat.create(model="grok-4.6", tools=tools_list)
            prompt = user_input
            chat.append(xai_sdk.chat.user(prompt))
        else:
            # Последующие сообщения ПРОСТО ДОБАВЛЯЮТСЯ в конец.
            # Это критично: если пересоздать chat или изменить старые сообщения,
            # кэш обнулится [citation:5].
            chat.append(xai_sdk.chat.user(user_input))

        # Внутренний цикл «мысль → действие → наблюдение»
        for turn in range(max_turns):
            response = chat.sample()
            stats.add_response(response)

            if response.tool_calls:
                for tool_call in response.tool_calls:
                    print(f"\n🧠 [Grok] Вызывает: {tool_call.function.name}")
                    import json
                    try:
                        args = json.loads(tool_call.function.arguments)
                        result = tool_handlers[tool_call.function.name](**args)
                    except Exception as e:
                        result = f"Ошибка выполнения: {e}"
                    # Результат инструмента ДОБАВЛЯЕТСЯ в конец.
                    # Кэшируемый префикс (всё до этого момента) остаётся неизменным [citation:18].
                    chat.append(xai_sdk.chat.tool_result(result))
            else:
                print(f"\n🤖 Grok: {response.content}")
                break
        else:
            print("\n⚠️ Достигнут лимит ходов.")

    stats.report()


namespace = invoke.Collection()
namespace.add_task(agent_loop)