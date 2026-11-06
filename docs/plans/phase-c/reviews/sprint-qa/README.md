# Phase C sprint QA reports

This snapshot was recorded on 2026-11-06T06:04:26.977644+00:00. Each report names its reviewed feature-branch commit and published PR comment.

This integration commit persists QA evidence only. It merges no sprint implementation and changes no anchor authority. Qualified source, final sprint acceptance and ordered integration remain separate recorded states.

| Sprint | QA pass | Verdict | Reviewed source | Report |
| --- | --- | --- | --- | --- |
| C-1 | qa-pr116-sprint-5 | IN-FLIGHT | `78cc2efef995a8780b1f7fdba99707bdb3a10a42` | [Report](c-1-latest.md) |
| C-2 | qa-pr119-sprint-4 | PASS | `b3e0b065d7d0d694b84e553abc393f9c0ad3e470` | [Report](c-2-latest.md) |
| C-3 | qa-pr120-sprint-4 | PASS | `5840414909f2d720e815956fd5247b0e23f5a52a` | [Report](c-3-latest.md) |
| C-4 | qa-pr122-sprint-3 | FAIL | `e190e2c8d84b9235ac70b3e442691980e8b69a33` | [Report](c-4-latest.md) |
| C-5 | qa-pr118-sprint-7 | IN-FLIGHT | `a44ab2069bc85c5cba0f0db234c5b37d4ca213a2` | [Report](c-5-latest.md) |
| C-6 | qa-pr121-sprint-3 | PASS | `b9216c955f43b08e9e5e5905868bbb323fcb363e` | [Report](c-6-latest.md) |

C2, C3 and C6 have qualified source and complete acceptance gates at their recorded commits. Their merge order remains governed by the Phase C plan. C1 awaits approved contract-stage fixture clarification; C5 retains its actual ANC-013 development-gate failure; C4 retains the policy and release/registry qualification findings listed in its report.

The central orchestration index preserves prior committed developer history and all 26 published sprint QA passes. Historical PASS records apply to their own source commits; the latest pointers above govern current status.

C4 QA2 and QA3 triage snapshots were persisted after source fix `e190e2c`. The C4 QA2 direct handoff was authorized before the fixes, and the formal fix assignment was rendered later. These records preserve that sequence and do not establish before-fix triage persistence.

Diagnostic logs, raw reviewer transcripts, test-result archives, calibration cases and prototypes are excluded. Canonical published PR comments remain the readback authority.
