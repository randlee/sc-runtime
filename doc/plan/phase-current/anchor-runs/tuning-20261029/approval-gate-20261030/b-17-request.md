Invoke the resolved `anchor-plan-review` definition at `.codex/agents/anchor-plan-review.md` with the harness **Agent** tool. Pass exactly this one fenced JSON params object and no prose:

```json
{
  "schema_version": "1.0.0",
  "runtime": {"agent":"anchor-plan-review","harness":"codex","model":"luna","path":".codex/agents/anchor-plan-review.md","sha256":"f1edd0f1c5b77787b44f7f75bf344eeabf73962dfc84597299240be0ff8ecafa","version":"0.8.5"},
  "operation": "plan_review",
  "correlation_id": "phase-b-approval-b17-20261030",
  "params": {"anchor_ids":"all","anchor_tracker":"bd","anchors_path":"/Volumes/Extreme Pro/github/sc-runtime-worktrees/plan/spinoff-monty-observability/doc/plan/phase-current/anchors.json","correlation_id":"phase-b-approval-b17-20261030","dry_run":false,"item_file":"/Volumes/Extreme Pro/github/sc-runtime-worktrees/plan/spinoff-monty-observability/docs/plans/phase-b/sprint-b-17-sc-cron-hook-environment.md","item_id":"b-17","item_title":"Sprint b-17 — shared Python/Monty hook environment","run_dir":"/Volumes/Extreme Pro/github/sc-runtime-worktrees/plan/spinoff-monty-observability/doc/plan/phase-current/anchor-runs/tuning-20261029/approval-gate-20261030","script":"/Volumes/Extreme Pro/github/sc-runtime-worktrees/plan/spinoff-monty-observability/.codex/skills/plan-anchoring/scripts/jev_judge.py","threshold":0.6}
}
```

Accept exactly one fenced JSON result. Save the response unchanged and validate
it with `anchor_runner_contract.py validate-fence --kind result --file
<response-file> --correlation-id <correlation-id>`; use its data only when it
reports `data.valid: true` (`data.valid: false` names the `violation`).