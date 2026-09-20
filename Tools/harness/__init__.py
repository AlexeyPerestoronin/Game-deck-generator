import os
import invoke
import pathlib
import datetime

from . import tools, agent, logger


@invoke.task()
def run_agent(ctx, prompt: str, iteration_limit: int = 25, safe_mode: bool = True):
    """Run Grok AI agent loop via invoke."""
    log_file = pathlib.Path(os.getcwd()) / ".log" / f"log-{datetime.datetime.now().strftime('%Y-%m-%d %H-%M')}.md"

    log = logger.DoubleLogger(pathlib.Path(log_file))
    ai_agent = agent.Grok(prompt, tools.DefaultToolsSet(safe_mode), log)
    agent_loop = agent.AgentLoop(iteration_limit, ai_agent, log)
    agent_loop.start_loop()


collection = invoke.Collection("harness")
collection.add_task(run_agent)
