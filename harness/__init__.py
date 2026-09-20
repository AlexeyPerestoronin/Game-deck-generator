import invoke

from . import (tools, agent, logger)

@invoke.task()
def agent_loop(ctx, prompt: str, iteration_limit: int = 25):
    logger = logger.DoubleLogger()
    ai_agent = agent.Grok(prompt, tools.DefaultToolsSet(), logger)
    agent_loop = agent.AgentLoop(iteration_limit, ai_agent, logger)
    agent_loop.start_loop()

namespace = invoke.Collection()
namespace.add_task(agent_loop)
