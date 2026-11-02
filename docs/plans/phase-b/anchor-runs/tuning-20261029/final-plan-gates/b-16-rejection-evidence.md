```json
{
  "schema_version": "1.0.0",
  "success": false,
  "canceled": false,
  "aborted_by": null,
  "data": null,
  "error": {
    "code": "POLICY.REMOTE_JUDGE_DISABLED",
    "message": "Automatic approval review rejected the single plan judge invocation because it would transmit internal anchor and sprint-plan contents to a remote Jev service, and the transcript did not specifically authorize exporting that payload to that destination. Do not bypass this rejection.",
    "recoverable": true,
    "suggested_action": "Obtain explicit approval to transmit the sprint plan and anchor contents to the remote Jev service, then rerun the review."
  },
  "metadata": {
    "duration_ms": 87000,
    "tool_calls": 36,
    "retry_count": 0,
    "correlation_id": "phase-b-final-b16-20261029"
  }
}
```
