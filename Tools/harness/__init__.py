import os
import json5
import invoke
import pathlib
import datetime

from . import logger, agent_loop


@invoke.task()
def run_loop(ctx, safe_mode: bool = False):
    """Run AI agent loop."""
    cwd = pathlib.Path(os.getcwd())

    with open(cwd / "harness.json5", "r", encoding="utf-8") as file:
        harness = json5.load(file)

    prompt_file = cwd / harness["task"]["dir"] / harness["task"]["prompt"]
    with open(prompt_file, "r", encoding="utf-8") as file:
        prompt = file.read()

    task_settings_file = cwd / harness["task"]["dir"] / harness["task"]["settings"]
    with open(task_settings_file, "r", encoding="utf-8") as file:
        agent_settings = json5.load(file)

    log_dir = cwd / harness["paths"]["log"] / f"{datetime.datetime.now().strftime('%Y-%m-%d %H-%M')}_{agent_settings['vendor']}_{agent_settings['model']}"
    log_dir.mkdir(exist_ok=False)
    log_file = log_dir / "log.md"
    dump_file = log_dir / "dump.json"
    loop = agent_loop.AgentLoop(safe_mode, logger.DoubleLogger(log_file), agent_settings)
    loop.start(prompt, dump_file)


collection = invoke.Collection("harness")
collection.add_task(run_loop)
