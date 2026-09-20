import os
import invoke
import pathlib
import datetime

from . import agents, logger, agent_loop


@invoke.task()
def run_agent(ctx, prompt: str, iteration_limit: int = 25, safe_mode: bool = True):
    """Run Grok AI agent loop via invoke."""
    log_file = pathlib.Path(os.getcwd()) / ".log" / f"log-{datetime.datetime.now().strftime('%Y-%m-%d %H-%M')}.md"

    log = logger.DoubleLogger(pathlib.Path(log_file))
    ai_agent = agents.Grok(prompt, agents.tools.DefaultTools(safe_mode), log)
    loop = agent_loop.AgentLoop(safe_mode, iteration_limit, ai_agent, log)
    loop.start()


collection = invoke.Collection("harness")
collection.add_task(run_agent)
