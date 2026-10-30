```json
{
  "schema_version": "1.0.0",
  "success": true,
  "canceled": false,
  "aborted_by": null,
  "data": {
    "gate": "plan",
    "item_id": "b-17",
    "run_id": "plan-20261030T013352Z-b-17",
    "run_path": "doc/plan/phase-current/anchor-runs/tuning-20261029/final-plan-gates/plan-20261030T013352Z-b-17.json",
    "dry_run": false,
    "reused_run": false,
    "pass": false,
    "failing": [
      "ANC-031"
    ],
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
        "near_threshold": true,
        "confirmed": true,
        "verdict": "underspecified",
        "answers": {
          "relevant": {
            "type": "choice",
            "choice": "yes",
            "confidence": 0.66,
            "probabilities": {
              "no": 0.22,
              "unclear": 0.01,
              "yes": 0.77
            }
          },
          "addressed": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.43,
            "probabilities": {
              "no": 0.62,
              "unclear": 0.03,
              "yes": 0.35
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.34,
            "probabilities": {
              "no": 0.56,
              "unclear": 0.13,
              "yes": 0.31
            }
          }
        },
        "min_confidence": 0.34,
        "explanation": "Jev was not confident (min confidence 0.34); the anchor or the bead may be underspecified.",
        "usage": {
          "input_tokens": 6022,
          "output_tokens": 117
        },
        "low_questions": [
          {
            "ask": "first",
            "role": "conflict",
            "question_id": "contradicted",
            "choice": "no",
            "confidence": 0.34
          }
        ],
        "attempts": 2,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23279,
        "to_fix": {
          "where": "Default feature builds exclude Monty and boundary lint permits only the declared types/dependencies.",
          "change": "State that boundary lint rejects production imports and dependencies on sc-compose and ATM while permitting test and benchmark references, and name the lint check that enforces this."
        }
      },
      {
        "anchor_id": "ANC-034",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "Python and Monty hooks use the same schedule-environment configuration and resolution rules.",
        "quote": "python/monty having consistent settings would be least confusing",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 2,
        "deterministic_check": "The same environment cases produce equivalent Python and Monty results.",
        "kind": "directive",
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
        },
        "min_confidence": 0.88,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6008,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23175
      },
      {
        "anchor_id": "ANC-035",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "Hooks inherit daemon-process environment values only through variable names explicitly allowed by daemon configuration.",
        "quote": "whitelisting api keys globally (by name) in config file (this is really a daemon scope decision)",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 2,
        "deterministic_check": "An allowed daemon value is delivered and an unlisted value is absent.",
        "kind": "directive",
        "leaning": "satisfied",
        "question_set": {
          "gate": "plan",
          "version": 1,
          "overridden": true
        },
        "questions_sha256": "f826ddd4cc1178bb42b5729fb97e85e6b3b2ab65498cde66ea0ee1dfa98a0a26",
        "questions": {
          "relevant": {
            "criteria": {
              "no": "The work is independent of the governed subject.",
              "unclear": "The work area is too vague to determine applicability.",
              "yes": "The work touches the governed subject directly or as a clear prerequisite."
            },
            "instructions": "Does item.text plan work on the governed activity, component, or artifact named by anchor.statement and anchor.scope? Judge applicability only, including dependencies.",
            "type": "choice"
          },
          "addressed": {
            "criteria": {
              "no": "The item omits the obligation or describes only a conflicting action.",
              "unclear": "The item gestures at the obligation but does not settle whether it is included.",
              "yes": "The required action or constraint is explicitly included."
            },
            "instructions": "Does item.text expressly include the specific obligation in anchor.statement as a planned step or constraint? A subject mention, silence, or opposing action does not satisfy the obligation.",
            "type": "choice"
          },
          "contradicted": {
            "criteria": {
              "no": "No incompatible behavior is proposed.",
              "unclear": "A possible conflict depends on an unresolved detail.",
              "yes": "The item commits to an incompatible behavior or boundary."
            },
            "instructions": "Does item.text commit to behavior that cannot coexist with anchor.statement? Silence, topical mention, and an expressly permitted exception are not conflicts.",
            "type": "choice"
          }
        },
        "verdict": "satisfied",
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
            "confidence": 0.98,
            "probabilities": {
              "no": 0.99,
              "unclear": 0.0,
              "yes": 0.01
            }
          }
        },
        "min_confidence": 0.98,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6030,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23342
      },
      {
        "anchor_id": "ANC-036",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "Each schedule can explicitly set or clear its own environment variables.",
        "quote": "having each entry having the ability to set or clear specific environment variables",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 2,
        "deterministic_check": "Two schedules set distinct identities and clearing removes a prior value.",
        "kind": "directive",
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
        },
        "min_confidence": 0.91,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6001,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23165
      },
      {
        "anchor_id": "ANC-037",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "Schedule creation and updates never implicitly capture environment values from the caller or daemon process.",
        "quote": "Never capture or persist inherited host values accidentally",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 3,
        "deterministic_check": "Ambient canaries are absent from persisted rows without explicit set input.",
        "kind": "prohibition",
        "leaning": "satisfied",
        "question_set": {
          "gate": "plan",
          "version": 1,
          "overridden": true
        },
        "questions_sha256": "f826ddd4cc1178bb42b5729fb97e85e6b3b2ab65498cde66ea0ee1dfa98a0a26",
        "questions": {
          "relevant": {
            "criteria": {
              "no": "The work is independent of the governed subject.",
              "unclear": "The work area is too vague to determine applicability.",
              "yes": "The work touches the governed subject directly or as a clear prerequisite."
            },
            "instructions": "Does item.text plan work on the governed activity, component, or artifact named by anchor.statement and anchor.scope? Judge applicability only, including dependencies.",
            "type": "choice"
          },
          "addressed": {
            "criteria": {
              "no": "The item omits the obligation or describes only a conflicting action.",
              "unclear": "The item gestures at the obligation but does not settle whether it is included.",
              "yes": "The required action or constraint is explicitly included."
            },
            "instructions": "Does item.text expressly include the specific obligation in anchor.statement as a planned step or constraint? A subject mention, silence, or opposing action does not satisfy the obligation.",
            "type": "choice"
          },
          "contradicted": {
            "criteria": {
              "no": "No incompatible behavior is proposed.",
              "unclear": "A possible conflict depends on an unresolved detail.",
              "yes": "The item commits to an incompatible behavior or boundary."
            },
            "instructions": "Does item.text commit to behavior that cannot coexist with anchor.statement? Silence, topical mention, and an expressly permitted exception are not conflicts.",
            "type": "choice"
          }
        },
        "verdict": "satisfied",
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
            "confidence": 0.98,
            "probabilities": {
              "no": 0.99,
              "unclear": 0.0,
              "yes": 0.01
            }
          }
        },
        "min_confidence": 0.98,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6031,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23320
      },
      {
        "anchor_id": "ANC-038",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "The schedule registry permits deliberately supplied environment values as plaintext configuration.",
        "quote": "why can we never add values?  all environment variables are not api keys?",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 2,
        "deterministic_check": "An explicit set value round-trips through the registry.",
        "kind": "directive",
        "leaning": "satisfied",
        "question_set": {
          "gate": "plan",
          "version": 1,
          "overridden": true
        },
        "questions_sha256": "e9965576f4baeafd046cab1602c342cfba185667897278270cdc80eb069acb36",
        "questions": {
          "relevant": {
            "criteria": {
              "no": "The item is about a different part of the system or a different activity, and completing it would neither follow nor break the anchor.",
              "unclear": "The item text is too vague to tell what part of the system it touches.",
              "yes": "The item plans, changes, or depends on the thing the anchor statement is about, directly or as a clear prerequisite."
            },
            "instructions": "The state holds one planning item (`item.title`, `item.text`) and one anchor: a directive the user stated verbatim in `anchor.statement`, with the area it governs in `anchor.scope`. Does the work described in `item.text` touch the subject that `anchor.statement` talks about? Judge overlap of subject matter only. Do not judge whether the item obeys the anchor.",
            "type": "choice"
          },
          "addressed": {
            "criteria": {
              "no": "The plan is silent or denies any permitted class, even if it mentions plaintext.",
              "unclear": "The plan names plaintext values but leaves the allowed class unresolved.",
              "yes": "The plan explicitly preserves the full permitted class of deliberate plaintext set values."
            },
            "instructions": "Does item.text expressly preserve the schedule registry permission to accept every deliberately supplied plaintext environment value, including secret and nonsecret values? Merely mentioning plaintext while banning a value class does not carry the permission forward.",
            "type": "choice"
          },
          "contradicted": {
            "criteria": {
              "no": "No incompatible restriction is planned.",
              "unclear": "The planned restriction or its applicability to deliberate set values is unresolved.",
              "yes": "The plan denies or narrows the anchor-permitted plaintext set-value capability."
            },
            "instructions": "Does item.text prohibit or narrow a class of deliberately supplied plaintext environment values that anchor.statement permits in the schedule registry? Treat an explicit secret-only ban as a conflict; silence and permitted handling are not conflicts.",
            "type": "choice"
          }
        },
        "verdict": "satisfied",
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
            "confidence": 0.97,
            "probabilities": {
              "no": 0.02,
              "unclear": 0.0,
              "yes": 0.98
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.94,
            "probabilities": {
              "no": 0.96,
              "unclear": 0.0,
              "yes": 0.04
            }
          }
        },
        "min_confidence": 0.94,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6143,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23913
      },
      {
        "anchor_id": "ANC-039",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "Schedules can resolve environment values through an application-provided resolver for opaque references.",
        "quote": "I also like allowing this:   \"OTHER_API_KEY\": { \"reference\": \"keychain://agents/planner-7/api-key\" }",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 2,
        "deterministic_check": "A consumer resolver supplies a referenced value without persisting the result.",
        "kind": "directive",
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
            "confidence": 0.97,
            "probabilities": {
              "no": 0.98,
              "unclear": 0.0,
              "yes": 0.02
            }
          }
        },
        "min_confidence": 0.97,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6004,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23183
      },
      {
        "anchor_id": "ANC-040",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "Schedules can load environment variables from full-path .env files.",
        "quote": "Another useful way to specify would be loading from a full-path .env file.",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 2,
        "deterministic_check": "An authorized absolute dotenv file supplies hook values.",
        "kind": "directive",
        "leaning": "satisfied",
        "question_set": {
          "gate": "plan",
          "version": 1,
          "overridden": true
        },
        "questions_sha256": "f826ddd4cc1178bb42b5729fb97e85e6b3b2ab65498cde66ea0ee1dfa98a0a26",
        "questions": {
          "relevant": {
            "criteria": {
              "no": "The work is independent of the governed subject.",
              "unclear": "The work area is too vague to determine applicability.",
              "yes": "The work touches the governed subject directly or as a clear prerequisite."
            },
            "instructions": "Does item.text plan work on the governed activity, component, or artifact named by anchor.statement and anchor.scope? Judge applicability only, including dependencies.",
            "type": "choice"
          },
          "addressed": {
            "criteria": {
              "no": "The item omits the obligation or describes only a conflicting action.",
              "unclear": "The item gestures at the obligation but does not settle whether it is included.",
              "yes": "The required action or constraint is explicitly included."
            },
            "instructions": "Does item.text expressly include the specific obligation in anchor.statement as a planned step or constraint? A subject mention, silence, or opposing action does not satisfy the obligation.",
            "type": "choice"
          },
          "contradicted": {
            "criteria": {
              "no": "No incompatible behavior is proposed.",
              "unclear": "A possible conflict depends on an unresolved detail.",
              "yes": "The item commits to an incompatible behavior or boundary."
            },
            "instructions": "Does item.text commit to behavior that cannot coexist with anchor.statement? Silence, topical mention, and an expressly permitted exception are not conflicts.",
            "type": "choice"
          }
        },
        "verdict": "satisfied",
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
            "confidence": 0.98,
            "probabilities": {
              "no": 0.99,
              "unclear": 0.0,
              "yes": 0.01
            }
          }
        },
        "min_confidence": 0.98,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6028,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23279
      },
      {
        "anchor_id": "ANC-041",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "Monty scripts can read their configured environment during source initialization, before hook functions run.",
        "quote": "there should also be the ability to set environment variables in a monty script before it runs.",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 2,
        "deterministic_check": "A top-level getenv reads the configured snapshot before cron_stop or cron_tick.",
        "kind": "directive",
        "leaning": "satisfied",
        "question_set": {
          "gate": "plan",
          "version": 1,
          "overridden": true
        },
        "questions_sha256": "f826ddd4cc1178bb42b5729fb97e85e6b3b2ab65498cde66ea0ee1dfa98a0a26",
        "questions": {
          "relevant": {
            "criteria": {
              "no": "The work is independent of the governed subject.",
              "unclear": "The work area is too vague to determine applicability.",
              "yes": "The work touches the governed subject directly or as a clear prerequisite."
            },
            "instructions": "Does item.text plan work on the governed activity, component, or artifact named by anchor.statement and anchor.scope? Judge applicability only, including dependencies.",
            "type": "choice"
          },
          "addressed": {
            "criteria": {
              "no": "The item omits the obligation or describes only a conflicting action.",
              "unclear": "The item gestures at the obligation but does not settle whether it is included.",
              "yes": "The required action or constraint is explicitly included."
            },
            "instructions": "Does item.text expressly include the specific obligation in anchor.statement as a planned step or constraint? A subject mention, silence, or opposing action does not satisfy the obligation.",
            "type": "choice"
          },
          "contradicted": {
            "criteria": {
              "no": "No incompatible behavior is proposed.",
              "unclear": "A possible conflict depends on an unresolved detail.",
              "yes": "The item commits to an incompatible behavior or boundary."
            },
            "instructions": "Does item.text commit to behavior that cannot coexist with anchor.statement? Silence, topical mention, and an expressly permitted exception are not conflicts.",
            "type": "choice"
          }
        },
        "verdict": "satisfied",
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
            "confidence": 0.98,
            "probabilities": {
              "no": 0.99,
              "unclear": 0.0,
              "yes": 0.01
            }
          }
        },
        "min_confidence": 0.98,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6032,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23327
      },
      {
        "anchor_id": "ANC-042",
        "source_type": "ANC",
        "source_ref": null,
        "statement": "The first released schedule schema establishes one baseline without requiring migrations among unreleased development schemas.",
        "quote": "i don't know why we are on schema v4 when we haven't even released binaries yet.",
        "stated_by": "randlee",
        "stated_at": "2026-10-29",
        "context": "Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected.",
        "anchor_version": 2,
        "deterministic_check": "First-release schema validation does not require a v1-v4 development migration ladder.",
        "kind": "directive",
        "leaning": "satisfied",
        "question_set": {
          "gate": "plan",
          "version": 1,
          "overridden": true
        },
        "questions_sha256": "f826ddd4cc1178bb42b5729fb97e85e6b3b2ab65498cde66ea0ee1dfa98a0a26",
        "questions": {
          "relevant": {
            "criteria": {
              "no": "The work is independent of the governed subject.",
              "unclear": "The work area is too vague to determine applicability.",
              "yes": "The work touches the governed subject directly or as a clear prerequisite."
            },
            "instructions": "Does item.text plan work on the governed activity, component, or artifact named by anchor.statement and anchor.scope? Judge applicability only, including dependencies.",
            "type": "choice"
          },
          "addressed": {
            "criteria": {
              "no": "The item omits the obligation or describes only a conflicting action.",
              "unclear": "The item gestures at the obligation but does not settle whether it is included.",
              "yes": "The required action or constraint is explicitly included."
            },
            "instructions": "Does item.text expressly include the specific obligation in anchor.statement as a planned step or constraint? A subject mention, silence, or opposing action does not satisfy the obligation.",
            "type": "choice"
          },
          "contradicted": {
            "criteria": {
              "no": "No incompatible behavior is proposed.",
              "unclear": "A possible conflict depends on an unresolved detail.",
              "yes": "The item commits to an incompatible behavior or boundary."
            },
            "instructions": "Does item.text commit to behavior that cannot coexist with anchor.statement? Silence, topical mention, and an expressly permitted exception are not conflicts.",
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
            "confidence": 0.91,
            "probabilities": {
              "no": 0.06,
              "unclear": 0.0,
              "yes": 0.94
            }
          },
          "contradicted": {
            "type": "choice",
            "choice": "no",
            "confidence": 0.97,
            "probabilities": {
              "no": 0.98,
              "unclear": 0.0,
              "yes": 0.02
            }
          }
        },
        "min_confidence": 0.91,
        "explanation": "The directive is carried forward.",
        "usage": {
          "input_tokens": 6034,
          "output_tokens": 117
        },
        "attempts": 1,
        "item_chars_sent": 21155,
        "item_shortened": false,
        "request_bytes": 23334
      }
    ],
    "overrides_applied": [
      "ANC-031",
      "ANC-034",
      "ANC-035",
      "ANC-036",
      "ANC-037",
      "ANC-038",
      "ANC-039",
      "ANC-040",
      "ANC-041",
      "ANC-042"
    ],
    "item_truncated": false,
    "shortened": [],
    "report_markdown": "## Anchor gate: plan · b-17 - Sprint b-17 — shared Python/Monty hook environment\n\nRun `plan-20261030T013352Z-b-17`, model `jev-1.14.0`, threshold 0.6. Counts: underspecified 1, satisfied 9.\n\n> **b-17 · ANC-031 · underspecified** Anchor: Every production release crate excludes sc-compose and ATM crate dependencies and Rust imports, enforced by boundary lint; test and benchmark references are permitted. You said (Phase B continuation discussion; all 16 statements approved, with atm_run mandatory and production dependency scope corrected., Oct 29): \"sc-compose and atm crates should be blocked from any production release crate.  sc-compose and atm can be referenced for tests i.e. benchmark tests.\" Jev was not confident (min confidence 0.34); the anchor or the bead may be underspecified. (leaning: satisfied)\n>   Low: conflict `contradicted` answered no at 0.34.\n\n"
  },
  "error": null,
  "metadata": {
    "duration_ms": 46000,
    "tool_calls": 22,
    "retry_count": 1,
    "correlation_id": "phase-b-final-b17-20261029"
  }
}
```
