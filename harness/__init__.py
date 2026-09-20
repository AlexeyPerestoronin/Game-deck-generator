import invoke

from . import (tools, agent, logger)

@invoke.task()
def run_agent(ctx, prompt: str, iteration_limit: int = 25):
    logger = logger.DoubleLogger()
    ai_agent = agent.Grok(prompt, tools.DefaultToolsSet(), logger)
    agent_loop = agent.AgentLoop(iteration_limit, ai_agent, logger)
    agent_loop.start_loop()

collection = invoke.Collection()
collection.add_task(run_agent)
