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

    with open(cwd / "agent-loop.json5", "r", encoding="utf-8") as file:
        loop_settings = json5.load(file)

    with open(cwd / loop_settings["task"] / "settings.json5", "r", encoding="utf-8") as file:
        agent_settings = json5.load(file)

    log = logger.DoubleLogger(cwd / loop_settings["paths"]["log"] / f"log-{datetime.datetime.now().strftime('%Y-%m-%d %H-%M')}.md")
    loop = agent_loop.AgentLoop(safe_mode, log, agent_settings)
    loop.start()


collection = invoke.Collection("harness")
collection.add_task(run_loop)
