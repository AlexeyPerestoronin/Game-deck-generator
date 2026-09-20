import invoke
import pathlib

from . import (tools, agent, logger)

@invoke.task()
def run_agent(ctx, prompt: str, log_file: str, iteration_limit: int = 25):
    logger = logger.DoubleLogger(pathlib.Path(log_file))
    ai_agent = agent.Grok(prompt, tools.DefaultToolsSet(), logger)
    agent_loop = agent.AgentLoop(iteration_limit, ai_agent, logger)
    agent_loop.start_loop()

collection = invoke.Collection()
collection.add_task(run_agent)
