```json
{
  "schema_version": "1.0.0",
  "success": true,
  "canceled": false,
  "aborted_by": null,
  "data": {
    "gate": "plan",
    "item_id": "b-16",
    "run_id": "plan-20261030T025051Z-b-16",
    "run_path": "/Volumes/Extreme Pro/github/sc-runtime-worktrees/plan/spinoff-monty-observability/doc/plan/phase-current/anchor-runs/tuning-20261029/approval-gate-20261030/plan-20261030T025051Z-b-16.json",
    "dry_run": false,
    "reused_run": false,
    "pass": false,
    "failing": [
      "ADR-RUN-0305",
      "ANC-034",
      "ANC-035",
      "ANC-037",
      "ANC-038",
      "ANC-039",
      "ANC-040",
      "ANC-041",
      "REQ-CRON-0002",
      "REQ-CRON-0004",
      "REQ-RUN-0315",
      "REQ-RUN-0317",
      "REQ-RUN-0318",
      "REQ-RUN-0320",
      "REQ-RUN-0322",
      "REQ-RUN-0325",
      "REQ-RUN-0326",
      "REQ-RUN-0327",
      "REQ-RUN-0328",
      "REQ-RUN-0329",
      "REQ-RUN-0331",
      "REQ-RUN-0503"
    ],
    "errors": [],
    "excluded": [],
    "results": [
      {
        "anchor_id": "ADR-RUN-0305",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.5,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.50); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.5,
            "probabilities": {
              "yes": 0.31,
              "no": 0.67,
              "unclear": 0.02
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.94,
            "probabilities": {
              "yes": 0.02,
              "no": 0.96,
              "unclear": 0.02
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.76,
            "probabilities": {
              "yes": 0.13,
              "no": 0.84,
              "unclear": 0.03
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.5
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id ADR-RUN-0305 --item b-16 --reason \"This sprint changes Monty host adapters and generated examples, not SQLite/Postgres row-copy adaptor design.\""
        }
      },
      {
        "anchor_id": "ANC-034",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.26,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.26); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.26,
            "probabilities": {
              "no": 0.51,
              "unclear": 0.04,
              "yes": 0.45
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.66,
            "probabilities": {
              "no": 0.77,
              "unclear": 0.09,
              "yes": 0.14
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.35,
            "probabilities": {
              "no": 0.57,
              "unclear": 0.36,
              "yes": 0.07
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.26
          },
          {
            "ask": "first",
            "role": "conflict",
            "question_id": "contradicted",
            "choice": "no",
            "confidence": 0.35
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id ANC-034 --item b-16 --reason \"The shared Python/Monty schedule-environment resolution contract is implemented by b-17, which owns the resolved environment snapshot.\""
        }
      },
      {
        "anchor_id": "ANC-035",
        "verdict": "underspecified",
        "leaning": "not_addressed",
        "min_confidence": 0.42,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.42); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.55,
            "probabilities": {
              "no": 0.27,
              "unclear": 0.03,
              "yes": 0.7
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.79,
            "probabilities": {
              "no": 0.86,
              "unclear": 0.06,
              "yes": 0.08
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.42,
            "probabilities": {
              "no": 0.61,
              "unclear": 0.12,
              "yes": 0.27
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.55
          },
          {
            "ask": "first",
            "role": "conflict",
            "question_id": "contradicted",
            "choice": "no",
            "confidence": 0.42
          }
        ],
        "to_fix": {
          "where": "b-17 adds read-only access to that run's resolved environment snapshot without changing handler registration or allowing process-global reads.",
          "change": "State explicitly that b-16 does not expose inherited daemon environment values to Monty hooks and that b-17 supplies only names allowed by daemon configuration."
        }
      },
      {
        "anchor_id": "ANC-037",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.35,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.35); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.35,
            "probabilities": {
              "no": 0.57,
              "unclear": 0.02,
              "yes": 0.41
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.88,
            "probabilities": {
              "no": 0.92,
              "unclear": 0.02,
              "yes": 0.06
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.53,
            "probabilities": {
              "no": 0.69,
              "unclear": 0.02,
              "yes": 0.29
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.35
          },
          {
            "ask": "first",
            "role": "conflict",
            "question_id": "contradicted",
            "choice": "no",
            "confidence": 0.53
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id ANC-037 --item b-16 --reason \"This sprint does not create or update schedules or persist caller or daemon environment values.\""
        }
      },
      {
        "anchor_id": "ANC-038",
        "verdict": "underspecified",
        "leaning": "not_addressed",
        "min_confidence": 0.4,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.40); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.4,
            "probabilities": {
              "no": 0.39,
              "unclear": 0.01,
              "yes": 0.6
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.96,
            "probabilities": {
              "no": 0.97,
              "unclear": 0.02,
              "yes": 0.01
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.52,
            "probabilities": {
              "no": 0.68,
              "unclear": 0.07,
              "yes": 0.25
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.4
          },
          {
            "ask": "first",
            "role": "conflict",
            "question_id": "contradicted",
            "choice": "no",
            "confidence": 0.52
          }
        ],
        "to_fix": {
          "where": "b-17 adds read-only access to that run's resolved environment snapshot without changing handler registration or allowing process-global reads.",
          "change": "Specify whether b-16 permits deliberately supplied plaintext schedule environment values, or state that this configuration behavior belongs to b-17."
        }
      },
      {
        "anchor_id": "ANC-039",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.44,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.44); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.58,
            "probabilities": {
              "no": 0.72,
              "unclear": 0.02,
              "yes": 0.26
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.44,
            "probabilities": {
              "no": 0.63,
              "unclear": 0.06,
              "yes": 0.31
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.66,
            "probabilities": {
              "no": 0.77,
              "unclear": 0.18,
              "yes": 0.05
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.58
          },
          {
            "ask": "first",
            "role": "compliance",
            "question_id": "addressed",
            "choice": "no",
            "confidence": 0.44
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id ANC-039 --item b-16 --reason \"Opaque environment-reference resolution is part of b-17 environment handling, not the Monty host registry or generated example delivered here.\""
        }
      },
      {
        "anchor_id": "ANC-040",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.53,
        "attempts": 2,
        "unstable": true,
        "explanation": "Jev was not confident (min confidence 0.53); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.53,
            "probabilities": {
              "no": 0.69,
              "unclear": 0.02,
              "yes": 0.29
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 1.0,
            "probabilities": {
              "no": 1.0,
              "unclear": 0.0,
              "yes": 0.0
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.97,
            "probabilities": {
              "no": 0.98,
              "unclear": 0.01,
              "yes": 0.01
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.53
          }
        ],
        "answers_second": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.61,
            "probabilities": {
              "no": 0.74,
              "unclear": 0.01,
              "yes": 0.25
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 1.0,
            "probabilities": {
              "no": 1.0,
              "unclear": 0.0,
              "yes": 0.0
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.96,
            "probabilities": {
              "no": 0.97,
              "unclear": 0.02,
              "yes": 0.01
            }
          }
        },
        "to_fix": {
          "where": null,
          "change": "exclude --id ANC-040 --item b-16 --reason \"Loading full-path .env files is not part of this sprint’s host-extension or template-example work.\""
        }
      },
      {
        "anchor_id": "ANC-041",
        "verdict": "underspecified",
        "leaning": "not_addressed",
        "min_confidence": 0.3,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.30); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.3,
            "probabilities": {
              "no": 0.45,
              "unclear": 0.02,
              "yes": 0.53
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.89,
            "probabilities": {
              "no": 0.93,
              "unclear": 0.05,
              "yes": 0.02
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.62,
            "probabilities": {
              "no": 0.75,
              "unclear": 0.06,
              "yes": 0.19
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.3
          }
        ],
        "to_fix": {
          "where": "b-17 adds read-only access to that run's resolved environment snapshot without changing handler registration or allowing process-global reads.",
          "change": "Specify whether Monty source initialization can read environment values before hooks run and require those reads to use the b-17 resolved snapshot rather than process-global state."
        }
      },
      {
        "anchor_id": "REQ-CRON-0002",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.46,
        "attempts": 1,
        "explanation": "Jev was not confident (min confidence 0.46); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.46,
            "probabilities": {
              "yes": 0.35,
              "no": 0.64,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.88,
            "probabilities": {
              "yes": 0.02,
              "no": 0.92,
              "unclear": 0.06
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.98,
            "probabilities": {
              "yes": 0.01,
              "no": 0.99,
              "unclear": 0.0
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.46
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id REQ-CRON-0002 --item b-16 --reason \"Cron expression parsing and five-field schedule semantics are not changed by this Monty host and template-example sprint.\""
        }
      },
      {
        "anchor_id": "REQ-CRON-0004",
        "verdict": "underspecified",
        "leaning": "not_addressed",
        "min_confidence": 0.35,
        "attempts": 1,
        "explanation": "Jev was not confident (min confidence 0.35); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.35,
            "probabilities": {
              "yes": 0.57,
              "no": 0.42,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.94,
            "probabilities": {
              "yes": 0.01,
              "no": 0.96,
              "unclear": 0.03
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.92,
            "probabilities": {
              "yes": 0.04,
              "no": 0.95,
              "unclear": 0.01
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.35
          }
        ],
        "to_fix": {
          "where": "3. REQ-RUN-0301 / REQ-CRON-0001 / ADR-RUN-0303: add optional daemon dependency `sc-cron`, direct `sc-observability-types`, and features equivalent to `cron = [\"dep:sc-cron\"]` and `monty = [\"cron\", \"sc-cron/monty\"]`; preserve the default feature set.",
          "change": "State that sc-cron creates only its own schedule table, does not migrate consumer tables, and allows schedule-row writes without constructing SqliteWriter."
        }
      },
      {
        "anchor_id": "REQ-RUN-0315",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.53,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.53); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.53,
            "probabilities": {
              "yes": 0.29,
              "no": 0.69,
              "unclear": 0.02
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.97,
            "probabilities": {
              "yes": 0.01,
              "no": 0.98,
              "unclear": 0.01
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.86,
            "probabilities": {
              "yes": 0.08,
              "no": 0.91,
              "unclear": 0.01
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.53
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id REQ-RUN-0315 --item b-16 --reason \"Host-aggregate storage adaptor topology is outside this sprint’s Monty host API and generated-example deliverables.\""
        }
      },
      {
        "anchor_id": "REQ-RUN-0317",
        "verdict": "underspecified",
        "leaning": "satisfied",
        "min_confidence": 0.38,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.38); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.55,
            "probabilities": {
              "yes": 0.7,
              "no": 0.29,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.38,
            "probabilities": {
              "yes": 0.59,
              "no": 0.37,
              "unclear": 0.04
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.88,
            "probabilities": {
              "yes": 0.07,
              "no": 0.92,
              "unclear": 0.01
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.55
          },
          {
            "ask": "first",
            "role": "compliance",
            "question_id": "addressed",
            "choice": "yes",
            "confidence": 0.38
          }
        ],
        "to_fix": {
          "where": "8. NFR-RUN-0005: add a generation test that renders the example and compiles the generated daemon with and without `monty`.",
          "change": "Add an explicit generation assertion that `just new` emits the adaptor Rust code and models for the generated project."
        }
      },
      {
        "anchor_id": "REQ-RUN-0318",
        "verdict": "underspecified",
        "leaning": "satisfied",
        "min_confidence": 0.3,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.30); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.3,
            "probabilities": {
              "yes": 0.53,
              "no": 0.46,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.73,
            "probabilities": {
              "yes": 0.12,
              "no": 0.82,
              "unclear": 0.06
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.5,
            "probabilities": {
              "yes": 0.31,
              "no": 0.67,
              "unclear": 0.02
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.3
          },
          {
            "ask": "first",
            "role": "conflict",
            "question_id": "contradicted",
            "choice": "no",
            "confidence": 0.5
          }
        ],
        "to_fix": {
          "where": "The template Cargo files are owned only by this sprint.",
          "change": "State that adaptor code generation remains in `just new` and that the `sc-runtime` library crate does not generate adaptor code."
        }
      },
      {
        "anchor_id": "REQ-RUN-0320",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.55,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.55); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.55,
            "probabilities": {
              "yes": 0.28,
              "no": 0.7,
              "unclear": 0.02
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.88,
            "probabilities": {
              "yes": 0.04,
              "no": 0.92,
              "unclear": 0.04
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.82,
            "probabilities": {
              "yes": 0.08,
              "no": 0.88,
              "unclear": 0.04
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.55
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id REQ-RUN-0320 --item b-16 --reason \"This sprint does not implement the copy or host-aggregate storage adaptor patterns across consumer repositories.\""
        }
      },
      {
        "anchor_id": "REQ-RUN-0322",
        "verdict": "underspecified",
        "leaning": "not_addressed",
        "min_confidence": 0.34,
        "attempts": 1,
        "explanation": "Jev was not confident (min confidence 0.34); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.73,
            "probabilities": {
              "yes": 0.82,
              "no": 0.17,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.34,
            "probabilities": {
              "yes": 0.39,
              "no": 0.56,
              "unclear": 0.05
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.4,
            "probabilities": {
              "yes": 0.38,
              "no": 0.6,
              "unclear": 0.02
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "compliance",
            "question_id": "addressed",
            "choice": "no",
            "confidence": 0.34
          },
          {
            "ask": "first",
            "role": "conflict",
            "question_id": "contradicted",
            "choice": "no",
            "confidence": 0.4
          }
        ],
        "to_fix": {
          "where": "Applications can register their own Monty host calls;",
          "change": "Clarify that this Monty host-call registry does not generate nonstandard storage adaptors; consumers build those adaptors from the store building blocks."
        }
      },
      {
        "anchor_id": "REQ-RUN-0325",
        "verdict": "underspecified",
        "leaning": "satisfied",
        "min_confidence": 0.28,
        "attempts": 1,
        "explanation": "Jev was not confident (min confidence 0.28); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.28,
            "probabilities": {
              "yes": 0.52,
              "no": 0.47,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.8,
            "probabilities": {
              "yes": 0.09,
              "no": 0.87,
              "unclear": 0.04
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.94,
            "probabilities": {
              "yes": 0.04,
              "no": 0.96,
              "unclear": 0.0
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.28
          }
        ],
        "to_fix": {
          "where": "The production-bridge benchmark and generated example invoke `sc_compose_run(\"render\", ...)` through the same optional adapter.",
          "change": "State that the generated example and runtime crates do not move rows between stores; that work remains in consumer-owned adaptors."
        }
      },
      {
        "anchor_id": "REQ-RUN-0326",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.44,
        "attempts": 1,
        "explanation": "Jev was not confident (min confidence 0.44); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.44,
            "probabilities": {
              "yes": 0.36,
              "no": 0.63,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.79,
            "probabilities": {
              "yes": 0.12,
              "no": 0.86,
              "unclear": 0.02
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.89,
            "probabilities": {
              "yes": 0.05,
              "no": 0.93,
              "unclear": 0.02
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.44
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id REQ-RUN-0326 --item b-16 --reason \"This sprint does not define the supported database-store combinations for a newly generated application.\""
        }
      },
      {
        "anchor_id": "REQ-RUN-0327",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.52,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.52); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.52,
            "probabilities": {
              "yes": 0.31,
              "no": 0.68,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.86,
            "probabilities": {
              "yes": 0.06,
              "no": 0.91,
              "unclear": 0.03
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.89,
            "probabilities": {
              "yes": 0.05,
              "no": 0.93,
              "unclear": 0.02
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.52
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id REQ-RUN-0327 --item b-16 --reason \"This sprint does not define whether two application stores share a schema.\""
        }
      },
      {
        "anchor_id": "REQ-RUN-0328",
        "verdict": "underspecified",
        "leaning": "not_addressed",
        "min_confidence": 0.43,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.43); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.53,
            "probabilities": {
              "yes": 0.69,
              "no": 0.29,
              "unclear": 0.02
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.43,
            "probabilities": {
              "yes": 0.31,
              "no": 0.62,
              "unclear": 0.07
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.72,
            "probabilities": {
              "yes": 0.1,
              "no": 0.81,
              "unclear": 0.09
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.53
          },
          {
            "ask": "first",
            "role": "compliance",
            "question_id": "addressed",
            "choice": "no",
            "confidence": 0.43
          }
        ],
        "to_fix": {
          "where": "Applications can register their own Monty host calls;",
          "change": "Clarify that Monty host extensibility leaves the consumer free to use a shared trait or distinct traits for its stores."
        }
      },
      {
        "anchor_id": "REQ-RUN-0329",
        "verdict": "underspecified",
        "leaning": "not_addressed",
        "min_confidence": 0.55,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.55); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.55,
            "probabilities": {
              "yes": 0.7,
              "no": 0.27,
              "unclear": 0.03
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.85,
            "probabilities": {
              "yes": 0.03,
              "no": 0.9,
              "unclear": 0.07
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.82,
            "probabilities": {
              "yes": 0.08,
              "no": 0.88,
              "unclear": 0.04
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "yes",
            "confidence": 0.55
          }
        ],
        "to_fix": {
          "where": "just new generated/test-monty --var-file wizard/fixtures/sqlite-mcp.json",
          "change": "Assert that generated crate package names use `<project>-<role>` and that the adaptor depends on `<project>-types` and `<project>-schema`."
        }
      },
      {
        "anchor_id": "REQ-RUN-0331",
        "verdict": "underspecified",
        "leaning": "not_applicable",
        "min_confidence": 0.55,
        "attempts": 2,
        "confirmed": true,
        "explanation": "Jev was not confident (min confidence 0.55); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.55,
            "probabilities": {
              "yes": 0.28,
              "no": 0.7,
              "unclear": 0.02
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.92,
            "probabilities": {
              "yes": 0.02,
              "no": 0.95,
              "unclear": 0.03
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.76,
            "probabilities": {
              "yes": 0.14,
              "no": 0.84,
              "unclear": 0.02
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "relevance",
            "question_id": "relevant",
            "choice": "no",
            "confidence": 0.55
          }
        ],
        "to_fix": {
          "where": null,
          "change": "exclude --id REQ-RUN-0331 --item b-16 --reason \"This sprint does not add or change the generated generic database command surface.\""
        }
      },
      {
        "anchor_id": "REQ-RUN-0503",
        "verdict": "underspecified",
        "leaning": "not_addressed",
        "min_confidence": 0.43,
        "attempts": 1,
        "explanation": "Jev was not confident (min confidence 0.43); the anchor or the bead may be underspecified.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.96,
            "probabilities": {
              "yes": 0.97,
              "no": 0.03,
              "unclear": 0.0
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.44,
            "probabilities": {
              "yes": 0.28,
              "no": 0.63,
              "unclear": 0.09
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.43,
            "probabilities": {
              "yes": 0.36,
              "no": 0.62,
              "unclear": 0.02
            }
          }
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "compliance",
            "question_id": "addressed",
            "choice": "no",
            "confidence": 0.44
          },
          {
            "ask": "first",
            "role": "conflict",
            "question_id": "contradicted",
            "choice": "no",
            "confidence": 0.43
          }
        ],
        "to_fix": {
          "where": "1. `REQ-CRON-0001`: `test_monty_example` and the two generated `cargo check` commands pass with the generated daemon as the only generated crate with the optional `sc-cron` edge; the feature-off build does not link it.",
          "change": "Require a freshly generated, unedited project to pass its lint and test commands plus both feature-off and Monty-enabled builds."
        }
      },
      {
        "anchor_id": "ADR-RUN-0002",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.89,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.98,
            "probabilities": {
              "yes": 0.99,
              "no": 0.01,
              "unclear": 0.0
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.89,
            "probabilities": {
              "yes": 0.93,
              "no": 0.05,
              "unclear": 0.02
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.98,
            "probabilities": {
              "yes": 0.01,
              "no": 0.99,
              "unclear": 0.0
            }
          }
        }
      },
      {
        "anchor_id": "ADR-RUN-0013",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.97,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "yes": 1.0,
              "no": 0.0,
              "unclear": 0.0
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.97,
            "probabilities": {
              "yes": 0.98,
              "no": 0.02,
              "unclear": 0.0
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.97,
            "probabilities": {
              "yes": 0.02,
              "no": 0.98,
              "unclear": 0.0
            }
          }
        }
      },
      {
        "anchor_id": "ANC-027",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.85,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "no": 0.0,
              "unclear": 0.0,
              "yes": 1.0
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.89,
            "probabilities": {
              "no": 0.06,
              "unclear": 0.01,
              "yes": 0.93
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.85,
            "probabilities": {
              "no": 0.9,
              "unclear": 0.01,
              "yes": 0.09
            }
          }
        }
      },
      {
        "anchor_id": "ANC-028",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.76,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "no": 0.0,
              "unclear": 0.0,
              "yes": 1.0
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
            "confidence": 0.76,
            "probabilities": {
              "no": 0.84,
              "unclear": 0.01,
              "yes": 0.15
            }
          }
        }
      },
      {
        "anchor_id": "ANC-029",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.91,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
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
            "confidence": 0.98,
            "probabilities": {
              "no": 0.01,
              "unclear": 0.0,
              "yes": 0.99
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.91,
            "probabilities": {
              "no": 0.94,
              "unclear": 0.01,
              "yes": 0.05
            }
          }
        }
      },
      {
        "anchor_id": "ANC-030",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.88,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "no": 0.0,
              "unclear": 0.0,
              "yes": 1.0
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
            "confidence": 0.88,
            "probabilities": {
              "no": 0.92,
              "unclear": 0.01,
              "yes": 0.07
            }
          }
        }
      },
      {
        "anchor_id": "ANC-031",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.89,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "no": 0.0,
              "unclear": 0.0,
              "yes": 1.0
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
            "confidence": 0.89,
            "probabilities": {
              "no": 0.93,
              "unclear": 0.0,
              "yes": 0.07
            }
          }
        }
      },
      {
        "anchor_id": "ANC-032",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.91,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.91,
            "probabilities": {
              "no": 0.06,
              "unclear": 0.0,
              "yes": 0.94
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.92,
            "probabilities": {
              "no": 0.05,
              "unclear": 0.0,
              "yes": 0.95
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.98,
            "probabilities": {
              "no": 0.99,
              "unclear": 0.0,
              "yes": 0.01
            }
          }
        }
      },
      {
        "anchor_id": "ANC-033",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.64,
        "attempts": 2,
        "confirmed": true,
        "explanation": "The directive is carried forward.",
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
            "confidence": 0.64,
            "probabilities": {
              "no": 0.76,
              "unclear": 0.01,
              "yes": 0.23
            }
          }
        }
      },
      {
        "anchor_id": "REQ-CRON-0001",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.97,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "yes": 1.0,
              "no": 0.0,
              "unclear": 0.0
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "yes": 1.0,
              "no": 0.0,
              "unclear": 0.0
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.97,
            "probabilities": {
              "yes": 0.02,
              "no": 0.98,
              "unclear": 0.0
            }
          }
        }
      },
      {
        "anchor_id": "REQ-CRON-0003",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.94,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "yes": 1.0,
              "no": 0.0,
              "unclear": 0.0
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.94,
            "probabilities": {
              "yes": 0.96,
              "no": 0.03,
              "unclear": 0.01
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.96,
            "probabilities": {
              "yes": 0.03,
              "no": 0.97,
              "unclear": 0.0
            }
          }
        }
      },
      {
        "anchor_id": "REQ-CRON-0005",
        "verdict": "satisfied",
        "leaning": "satisfied",
        "min_confidence": 0.94,
        "attempts": 1,
        "explanation": "The directive is carried forward.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 1.0,
            "probabilities": {
              "yes": 1.0,
              "no": 0.0,
              "unclear": 0.0
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.94,
            "probabilities": {
              "yes": 0.96,
              "no": 0.03,
              "unclear": 0.01
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.98,
            "probabilities": {
              "yes": 0.01,
              "no": 0.99,
              "unclear": 0.0
            }
          }
        }
      },
      {
        "anchor_id": "ANC-036",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.58,
        "attempts": 2,
        "confirmed": true,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.62,
            "probabilities": {
              "no": 0.75,
              "unclear": 0.02,
              "yes": 0.23
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.68,
            "probabilities": {
              "no": 0.79,
              "unclear": 0.05,
              "yes": 0.16
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.58,
            "probabilities": {
              "no": 0.72,
              "unclear": 0.14,
              "yes": 0.14
            }
          }
        }
      },
      {
        "anchor_id": "ANC-042",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.77,
        "attempts": 1,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.77,
            "probabilities": {
              "no": 0.85,
              "unclear": 0.01,
              "yes": 0.14
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.92,
            "probabilities": {
              "no": 0.95,
              "unclear": 0.01,
              "yes": 0.04
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.88,
            "probabilities": {
              "no": 0.92,
              "unclear": 0.02,
              "yes": 0.06
            }
          }
        }
      },
      {
        "anchor_id": "REQ-RUN-0313",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.77,
        "attempts": 1,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.88,
            "probabilities": {
              "yes": 0.07,
              "no": 0.92,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.91,
            "probabilities": {
              "yes": 0.04,
              "no": 0.94,
              "unclear": 0.02
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.77,
            "probabilities": {
              "yes": 0.12,
              "no": 0.85,
              "unclear": 0.03
            }
          }
        }
      },
      {
        "anchor_id": "REQ-RUN-0314",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.72,
        "attempts": 1,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.72,
            "probabilities": {
              "yes": 0.18,
              "no": 0.81,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 1.0,
            "probabilities": {
              "yes": 0.0,
              "no": 1.0,
              "unclear": 0.0
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.96,
            "probabilities": {
              "yes": 0.03,
              "no": 0.97,
              "unclear": 0.0
            }
          }
        }
      },
      {
        "anchor_id": "REQ-RUN-0316",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.77,
        "attempts": 1,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.82,
            "probabilities": {
              "yes": 0.1,
              "no": 0.88,
              "unclear": 0.02
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.97,
            "probabilities": {
              "yes": 0.01,
              "no": 0.98,
              "unclear": 0.01
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.77,
            "probabilities": {
              "yes": 0.11,
              "no": 0.85,
              "unclear": 0.04
            }
          }
        }
      },
      {
        "anchor_id": "REQ-RUN-0319",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.91,
        "attempts": 1,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.91,
            "probabilities": {
              "yes": 0.05,
              "no": 0.94,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 1.0,
            "probabilities": {
              "yes": 0.0,
              "no": 1.0,
              "unclear": 0.0
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.96,
            "probabilities": {
              "yes": 0.02,
              "no": 0.97,
              "unclear": 0.01
            }
          }
        }
      },
      {
        "anchor_id": "REQ-RUN-0321",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.85,
        "attempts": 1,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.85,
            "probabilities": {
              "yes": 0.09,
              "no": 0.9,
              "unclear": 0.01
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.92,
            "probabilities": {
              "yes": 0.01,
              "no": 0.95,
              "unclear": 0.04
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.86,
            "probabilities": {
              "yes": 0.07,
              "no": 0.91,
              "unclear": 0.02
            }
          }
        }
      },
      {
        "anchor_id": "REQ-RUN-0323",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.7,
        "attempts": 2,
        "confirmed": true,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.7,
            "probabilities": {
              "yes": 0.18,
              "no": 0.8,
              "unclear": 0.02
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.94,
            "probabilities": {
              "yes": 0.01,
              "no": 0.96,
              "unclear": 0.03
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.89,
            "probabilities": {
              "yes": 0.05,
              "no": 0.93,
              "unclear": 0.02
            }
          }
        }
      },
      {
        "anchor_id": "REQ-RUN-0324",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.6,
        "attempts": 2,
        "confirmed": true,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.6,
            "probabilities": {
              "yes": 0.25,
              "no": 0.73,
              "unclear": 0.02
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.94,
            "probabilities": {
              "yes": 0.01,
              "no": 0.96,
              "unclear": 0.03
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.79,
            "probabilities": {
              "yes": 0.13,
              "no": 0.86,
              "unclear": 0.01
            }
          }
        }
      },
      {
        "anchor_id": "REQ-RUN-0330",
        "verdict": "not_applicable",
        "leaning": "not_applicable",
        "min_confidence": 0.76,
        "attempts": 1,
        "explanation": "The anchor does not govern this bead.",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.76,
            "probabilities": {
              "yes": 0.13,
              "no": 0.84,
              "unclear": 0.03
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.97,
            "probabilities": {
              "yes": 0.01,
              "no": 0.98,
              "unclear": 0.01
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.94,
            "probabilities": {
              "yes": 0.02,
              "no": 0.96,
              "unclear": 0.02
            }
          }
        }
      }
    ],
    "overrides_applied": [
      "ANC-034",
      "ANC-035",
      "ANC-037",
      "ANC-038",
      "ANC-039",
      "ANC-040",
      "ANC-041",
      "ANC-027",
      "ANC-028",
      "ANC-029",
      "ANC-030",
      "ANC-031",
      "ANC-032",
      "ANC-033",
      "ANC-036",
      "ANC-042"
    ],
    "item_truncated": false,
    "shortened": [],
    "report_markdown": "## Anchor gate: plan · b-16 - Sprint b-16 — extensible Monty hosts and generated example\n\nRun `plan-20261030T025051Z-b-16`, model `jev-1.14.0`, threshold 0.6. Counts: underspecified 22, satisfied 12, not_applicable 10.\n\n> **b-16 · ADR-RUN-0305 · underspecified** Anchor: SQLite and Postgres meet in an adaptor crate generated into the consumer repository. Jev was not confident (min confidence 0.50); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.50.\n\n> **b-16 · ANC-034 · underspecified** Anchor: Python and Monty hooks use the same schedule-environment configuration and resolution rules. You said (Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., Oct 29): \"python/monty having consistent settings would be least confusing.\" Jev was not confident (min confidence 0.26); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.26.\n>   Low: conflict `contradicted` answered no at 0.35.\n\n> **b-16 · ANC-035 · underspecified** Anchor: Hooks inherit daemon-process environment values only through variable names explicitly allowed by daemon configuration. You said (Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., Oct 29): \"whitelisting api keys globally (by name) in config file (this is really a daemon scope decision).\" Jev was not confident (min confidence 0.42); the anchor or the bead may be underspecified. (leaning: not_addressed)\n>   Low: relevance `relevant` answered yes at 0.55.\n>   Low: conflict `contradicted` answered no at 0.42.\n\n> **b-16 · ANC-037 · underspecified** Anchor: Schedule creation and updates never implicitly capture environment values from the caller or daemon process. You said (Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., Oct 29): \"Never capture or persist inherited host values accidentally.\" Jev was not confident (min confidence 0.35); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.35.\n>   Low: conflict `contradicted` answered no at 0.53.\n\n> **b-16 · ANC-038 · underspecified** Anchor: The schedule registry permits deliberately supplied environment values as plaintext configuration. You said (Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., Oct 29): \"why can we never add values?  all environment variables are not api keys?\" Jev was not confident (min confidence 0.40); the anchor or the bead may be underspecified. (leaning: not_addressed)\n>   Low: relevance `relevant` answered yes at 0.40.\n>   Low: conflict `contradicted` answered no at 0.52.\n\n> **b-16 · ANC-039 · underspecified** Anchor: Schedules can resolve environment values through an application-provided resolver for opaque references. You said (Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., Oct 29): \"I also like allowing this:   \"OTHER_API_KEY\": { \"reference\": \"keychain://agents/planner-7/api-key\" }.\" Jev was not confident (min confidence 0.44); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.58.\n>   Low: compliance `addressed` answered no at 0.44.\n\n> **b-16 · ANC-040 · underspecified** Anchor: Schedules can load environment variables from full-path .env files. You said (Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., Oct 29): \"Another useful way to specify would be loading from a full-path .env file.\" Jev was not confident (min confidence 0.53); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.53.\n>   Unstable: two identical requests gave underspecified then not_applicable.\n\n> **b-16 · ANC-041 · underspecified** Anchor: Monty scripts can read their configured environment during source initialization, before hook functions run. You said (Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., Oct 29): \"there should also be the ability to set environment variables in a monty script before it runs.\" Jev was not confident (min confidence 0.30); the anchor or the bead may be underspecified. (leaning: not_addressed)\n>   Low: relevance `relevant` answered yes at 0.30.\n\n> **b-16 · REQ-CRON-0002 · underspecified** Anchor: A sc-cron schedule is a normal crontab string of five fields: minute, hour, day of month, month, and day of week. REQ-CRON-0002 (requirements register) says: \"CronExpr::parse of */5 * * * * succeeds and means every fifth minute. CronExpr::parse of 0 */5 * * * * succeeds and means second 0 of every fifth minute. CronExpr::parse of @hourly succeeds.\" Jev was not confident (min confidence 0.46); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.46.\n\n> **b-16 · REQ-CRON-0004 · underspecified** Anchor: The sc-cron crate creates one schedule table. A test writes a row without constructing SqliteWriter. The crate does not migrate the consumer's other tables. Jev was not confident (min confidence 0.35); the anchor or the bead may be underspecified. (leaning: not_addressed)\n>   Low: relevance `relevant` answered yes at 0.35.\n\n> **b-16 · REQ-RUN-0315 · underspecified** Anchor: The host-aggregate adaptor is one SQLite database for this host and a Postgres database that aggregates many hosts. Jev was not confident (min confidence 0.53); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.53.\n\n> **b-16 · REQ-RUN-0317 · underspecified** Anchor: Adaptor Rust code and models in a new repository are generated by the command just new (scripts/new_project.py). Jev was not confident (min confidence 0.38); the anchor or the bead may be underspecified. (leaning: satisfied)\n>   Low: relevance `relevant` answered yes at 0.55.\n>   Low: compliance `addressed` answered yes at 0.38.\n\n> **b-16 · REQ-RUN-0318 · underspecified** Anchor: The library crate named sc-runtime does not generate adaptor code. Jev was not confident (min confidence 0.30); the anchor or the bead may be underspecified. (leaning: satisfied)\n>   Low: relevance `relevant` answered yes at 0.30.\n>   Low: conflict `contradicted` answered no at 0.50.\n\n> **b-16 · REQ-RUN-0320 · underspecified** Anchor: Every consumer repository uses the same generated implementation of the copy pattern and the same generated implementation of the host-aggregate pattern. Jev was not confident (min confidence 0.55); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.55.\n\n> **b-16 · REQ-RUN-0322 · underspecified** Anchor: An adaptor that is neither the copy pattern nor the host-aggregate pattern is written by the consumer from the store building blocks. Jev was not confident (min confidence 0.34); the anchor or the bead may be underspecified. (leaning: not_addressed)\n>   Low: compliance `addressed` answered no at 0.34.\n>   Low: conflict `contradicted` answered no at 0.40.\n\n> **b-16 · REQ-RUN-0325 · underspecified** Anchor: The sc-runtime crates do not move rows between stores. Jev was not confident (min confidence 0.28); the anchor or the bead may be underspecified. (leaning: satisfied)\n>   Low: relevance `relevant` answered yes at 0.28.\n\n> **b-16 · REQ-RUN-0326 · underspecified** Anchor: A new application is created with no database, with SQLite only, with one sqlx SQL engine only, or with SQLite plus one sqlx SQL engine. Jev was not confident (min confidence 0.44); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.44.\n\n> **b-16 · REQ-RUN-0327 · underspecified** Anchor: Two stores in one application use the same schema or a similar schema. Jev was not confident (min confidence 0.52); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.52.\n\n> **b-16 · REQ-RUN-0328 · underspecified** Anchor: Two stores in one application use one shared trait or a different trait per store. Jev was not confident (min confidence 0.43); the anchor or the bead may be underspecified. (leaning: not_addressed)\n>   Low: relevance `relevant` answered yes at 0.53.\n>   Low: compliance `addressed` answered no at 0.43.\n\n> **b-16 · REQ-RUN-0329 · underspecified** Anchor: A generated crate's package name is the project name, a hyphen, and its role. REQ-RUN-0329 (requirements register) says: \"A generated crate's package name is the project name, a hyphen, and its role. The adaptor depends on <project>-types and <project>-schema.\" Jev was not confident (min confidence 0.55); the anchor or the bead may be underspecified. (leaning: not_addressed)\n>   Low: relevance `relevant` answered yes at 0.55.\n\n> **b-16 · REQ-RUN-0331 · underspecified** Anchor: The template ships general database commands on the CLI, MCP, and HTTP, including delete row <table> <id>. The consumer can disable that surface. Jev was not confident (min confidence 0.55); the anchor or the bead may be underspecified. (leaning: not_applicable)\n>   Low: relevance `relevant` answered no at 0.55.\n\n> **b-16 · REQ-RUN-0503 · underspecified** Anchor: A project generated by just new passes lint, test, and both builds with no file edited. Jev was not confident (min confidence 0.43); the anchor or the bead may be underspecified. (leaning: not_addressed)\n>   Low: compliance `addressed` answered no at 0.44.\n>   Low: conflict `contradicted` answered no at 0.43.\n\n"
  },
  "error": null,
  "metadata": {
    "duration_ms": 73000,
    "tool_calls": 36,
    "retry_count": 20,
    "correlation_id": "phase-b-approval-b16-20261030"
  }
}
```
