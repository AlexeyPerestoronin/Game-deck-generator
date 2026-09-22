import os
import json5
import invoke
import pathlib
import datetime

from . import logger, agent_loop


@invoke.task()
def run_loop(ctx, safe_mode: bool = False):
    """Run Gemini-3.8-Flash AI agent loop via invoke."""
    cwd = pathlib.Path(os.getcwd())

    with open(cwd / "harness.json5", "r", encoding="utf-8") as file:
        harness = json5.load(file)

    prompt_file = cwd / harness["task"]["prompt"] / harness["task"]["prompt"]
    with open(prompt_file, "r", encoding="utf-8") as file:
        prompt = json5.load(file)

    task_settings_file = cwd / harness["task"]["dir"] / harness["task"]["settings"]
    with open(task_settings_file, "r", encoding="utf-8") as file:
        agent_settings = json5.load(file)

    log = logger.DoubleLogger(cwd / harness["paths"]["log"] / f"log-{datetime.datetime.now().strftime('%Y-%m-%d %H-%M')}.md")
    loop = agent_loop.AgentLoop(safe_mode, log, agent_settings)
    loop.start(prompt)


collection = invoke.Collection("harness")
collection.add_task(run_loop)
