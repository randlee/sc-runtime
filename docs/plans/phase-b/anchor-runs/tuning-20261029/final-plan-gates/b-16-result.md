```json
{
  "schema_version": "1.0.0",
  "success": false,
  "canceled": false,
  "aborted_by": null,
  "data": null,
  "error": {
    "code": "POLICY.REMOTE_JUDGE_DISABLED",
    "message": "Automatic approval review rejected the remote Jev judgment because it would transmit internal anchors and sprint-plan contents. The reviewer treated the repository setting as untrusted evidence and said the user's general Jev-review request did not specifically authorize sending this payload to that destination. Do not bypass this rejection.",
    "recoverable": true,
    "suggested_action": "Obtain approval that specifically authorizes sending this sprint plan and its anchors to the remote Jev service."
  },
  "metadata": {
    "duration_ms": 12000,
    "tool_calls": 1,
    "retry_count": 0,
    "correlation_id": "phase-b-final-b16-20261029"
  }
}
```
