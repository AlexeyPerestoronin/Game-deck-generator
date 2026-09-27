import json5
import invoke
import pathlib
import datetime

from . import logger, agent_loop


def _get_log_folder(settings) -> pathlib.Path:
    log_folder_1part = pathlib.Path(settings["cwd"]) / pathlib.Path(settings["paths"]["log-folder"])
    log_folder_2part = f"{datetime.datetime.now().strftime('%Y-%m-%d %H-%M')}_{settings['vendor']}_{settings['model']}"
    log_folder = log_folder_1part / log_folder_2part
    log_folder.mkdir(parents=True, exist_ok=True)
    return log_folder

@invoke.task()
def run_loop(ctx, settings: str):
    """Run AI agent loop."""
    with open(pathlib.Path(settings), "r", encoding="utf-8") as file:
        settings = json5.load(file)

    cwd = pathlib.Path(settings["cwd"])
    safe_mode = True if settings["mode"] == "auto accept" else False

    log_dir = _get_log_folder(settings)
    log_file = log_dir / "log.md"
    dump_file = log_dir / "dump.json"
    settings["tool-settings"]["cwd"] = cwd
    settings["tool-settings"]["temp-dir"] = log_dir
    loop = agent_loop.AgentLoop(safe_mode, logger.DoubleLogger(log_file), settings)
    loop.start(settings["prompt"], dump_file)


collection = invoke.Collection("harness")
collection.add_task(run_loop)
