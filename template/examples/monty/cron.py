"""CLI hosts share the daemon/files/schedule environment snapshot.

The application sets ATM_IDENTITY/ATM_TEAM, clears OPENAI_API_KEY and resolves
OTHER_API_KEY via its recording resolver; the optional absolute dotenv supplies
AGENT_MODE. No source reads the daemon environment implicitly.
"""

import os

INITIAL_IDENTITY = os.getenv("ATM_IDENTITY", "unconfigured")


def cron_stop(hook):
    return {"action": "continue", "reason": "application remains enabled"}


def cron_tick(hook):
    payload = hook["payload"]
    if os.getenv("ATM_IDENTITY") != payload["identity"]:
        raise RuntimeError("schedule identity was not delivered")
    if os.getenv("ATM_TEAM") != payload["team"]:
        raise RuntimeError("schedule team was not delivered")
    if os.getenv("OPENAI_API_KEY") is not None:
        raise RuntimeError("cleared environment name is present")
    if os.getenv("OTHER_API_KEY") is None:
        raise RuntimeError("application reference was not resolved")
    if os.getenv("AGENT_MODE") != "recording":
        raise RuntimeError("dotenv layer was not delivered")
    composed = sc_compose_run("render", payload["compose_args"])
    agents = atm_run("agent", ["list"])
    if composed["exit_code"] != 0 or agents["exit_code"] != 0:
        raise RuntimeError("application CLI command failed")
    return None
