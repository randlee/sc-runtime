```json
{
  "schema_version": "1.0.0",
  "success": true,
  "canceled": false,
  "aborted_by": null,
  "data": {
    "gate": "plan",
    "item_id": "b-13",
    "run_id": "plan-20261030T013109Z-b-13",
    "run_path": "doc/plan/phase-current/anchor-runs/tuning-20261029/final-plan-gates/plan-20261030T013109Z-b-13.json",
    "dry_run": false,
    "reused_run": true,
    "pass": true,
    "failing": [],
    "errors": [],
    "excluded": [],
    "results": [
      {
        "anchor_id": "ANC-031",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "Every production release crate excludes sc-compose and ATM crate dependencies and Rust imports, enforced by boundary lint; test and benchmark references are permitted.",
        "quote": "sc-compose and atm crates should be blocked from any production release crate.  sc-compose and atm can be referenced for tests i.e. benchmark tests",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 3,
        "deterministic_check": "Boundary lint rejects production imports/dependencies while permitting test-only references.",
        "kind": "prohibition",
        "leaning": "satisfied",
        "question_set": {
          "gate": "plan",
          "version": 1,
          "overridden": true
        },
        "questions_sha256": "f757ba41a54ab1c5b169ce1c4bc3b6479d9df3d9d5d5e711725f709fe90bed70",
        "questions": {
          "relevant": {
            "criteria": {
              "no": "The anchor does not apply to this item.",
              "unclear": "The item does not identify enough work to decide applicability.",
              "yes": "The anchor applies to this item."
            },
            "instructions": "Would completing item.text exercise, change, or depend on the subject of anchor.statement? Do not use compliance as evidence of relevance.",
            "type": "choice"
          },
          "addressed": {
            "criteria": {
              "no": "The item is silent, merely topical, or includes only the opposite behavior.",
              "unclear": "The text might include the requirement but is too indefinite to tell.",
              "yes": "The item explicitly plans or preserves the required behavior or boundary."
            },
            "instructions": "Is the requirement itself carried into the plan item, using equivalent words if needed? Judge its presence independently of contradiction elsewhere.",
            "type": "choice"
          },
          "contradicted": {
            "criteria": {
              "no": "The item can coexist with the anchor, including when silent.",
              "unclear": "The item leaves a potentially decisive choice unspecified.",
              "yes": "A stated plan would violate the anchor."
            },
            "instructions": "Compare the proposed item with the anchor obligation for an explicit incompatible choice. Treat allowed exceptions as allowed.",
            "type": "choice"
          }
        },
        "verdict": "satisfied",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.98,
            "probabilities": {
              "no": 0.01,
              "unclear": 0.0,
              "yes": 0.99
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "no": 0.0,
              "unclear": 0.0,
              "yes": 1.0
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.86,
            "probabilities": {
              "no": 0.91,
              "unclear": 0.01,
              "yes": 0.08
            }
          }
        },
        "min_confidence": 0.86,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6701,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21276,
        "item_shortened": false,
        "request_bytes": 23426
      }
    ],
    "overrides_applied": [
      "ANC-031"
    ],
    "item_truncated": false,
    "shortened": [],
    "report_markdown": "## Anchor gate: plan · b-13 - Sprint b-13 — observability injection contract and adoption\n\nRun `plan-20261030T013109Z-b-13`, model `jev-1.14.0`, threshold 0.6. Counts: satisfied 1.\n\nAll judged anchors are satisfied or not applicable."
  },
  "error": null,
  "metadata": {
    "duration_ms": 66,
    "tool_calls": 1,
    "retry_count": 0,
    "correlation_id": "phase-b-final-b13-20261029"
  }
}
```
