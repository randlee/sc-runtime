"""Opt-in application-registered CLI hosts; payload supplies ordinary argv."""


def cron_stop(hook):
    return {"action": "continue", "reason": "application remains enabled"}


def cron_tick(hook):
    payload = hook["payload"]
    composed = sc_compose_run("render", payload["compose_args"])
    agents = atm_run("agent", ["list"])
    if composed["exit_code"] != 0 or agents["exit_code"] != 0:
        raise RuntimeError("application CLI command failed")
    return None
