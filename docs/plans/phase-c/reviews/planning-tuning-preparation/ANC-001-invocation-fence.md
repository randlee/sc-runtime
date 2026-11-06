```json
{
  "schema_version": "1.0.0",
  "runtime": {
    "agent": "anchor-tuning",
    "harness": "codex",
    "model": "sol",
    "path": ".codex/agents/anchor-tuning.md",
    "sha256": "222ef5fe7ef544511346c93a86239f9e1112e4fbe09fb9e81c373f46a584cc4a",
    "version": "0.8.6"
  },
  "operation": "tune",
  "correlation_id": "sc-runtime-phase-c-anc-001-plan-clarity-tune-r1",
  "params": {
    "schema_version": "2.0.0",
    "operation": "tune",
    "template": {
      "id": "anchor-tuning-tune-request",
      "version": "0.4.1",
      "sha256": "d8d6796121bb8155af53277053b25fe9cf99ead86ba55f51b19df8962b441ed3"
    },
    "correlation_id": "sc-runtime-phase-c-anc-001-plan-clarity-tune-r1",
    "runtime": {
      "workspace": "/Users/randlee/github/sc-runtime-worktrees/plan/phase-c",
      "script": "/Users/randlee/github/sc-runtime-worktrees/plan/phase-c/.codex/skills/plan-anchoring/scripts/jev_judge.py",
      "version": "0.8.6",
      "sha256": "be99ae73699f25d5385e3611461eb5d874cd9ce72d7c451995869f09fa3012de",
      "contract_script": "/Users/randlee/github/sc-runtime-worktrees/plan/phase-c/.codex/skills/plan-anchoring/scripts/anchor_tuning_contract.py",
      "contract_sha256": "c83a07f2e36d260a3c7e35759bfbabbc6d283167eb488711629b756d79388be4",
      "agent": "anchor-tuning",
      "harness": "codex",
      "model": "sol"
    },
    "authority": {
      "intent": {
        "id": "sc-runtime-phase-c-review-completion-intent",
        "version": 1,
        "content": "I need you working through a checklist so that all of these items are completed",
        "sha256": "6bbac0c410825010cc8da531e7cc4cecdd1b15dfb4752d6a371527874385264e"
      },
      "anchors": {
        "sha256": "e2eef4b9b1a90d7af0725b8508ec2eb4bae68614a8727a4e683787a707fe1c71",
        "records": [
          {
            "id": "ANC-001",
            "version": 4,
            "state": "locked",
            "statement": "Generated consumer setup must install sc-lint with its sc-boundary, sc-portability, and sc-runtime analyzers.",
            "kind": "directive",
            "scope": "c-2 generated consumer setup and any plan item that installs the consumer lint toolchain.",
            "check": "A generated fixture runs just setup, ordinary just lint, and named default and full profiles while its receipt records sc-boundary, sc-portability, and sc-runtime.",
            "fixtures": {
              "pass": [
                "Consumer setup installs the full sc-lint toolchain, including its boundary, portability, and runtime analyzers."
              ],
              "fail": [
                "Consumer setup installs only rustfmt and clippy and does not install the sc-lint toolchain."
              ]
            },
            "quote": "a full sc-lint is installed on a consumer repo"
          }
        ]
      }
    },
    "gate": "planning",
    "current_questions": {
      "relevant": {
        "type": "choice",
        "instructions": "The state holds one planning item (`item.title`, `item.text`) and one anchor: a directive the user stated verbatim in `anchor.statement`, with the area it governs in `anchor.scope`. Does the work described in `item.text` touch the subject that `anchor.statement` talks about? Judge overlap of subject matter only. Do not judge whether the item obeys the anchor.",
        "criteria": {
          "yes": "The item plans, changes, or depends on the thing the anchor statement is about, directly or as a clear prerequisite.",
          "no": "The item is about a different part of the system or a different activity, and completing it would neither follow nor break the anchor.",
          "unclear": "The item text is too vague to tell what part of the system it touches."
        }
      },
      "addressed": {
        "type": "choice",
        "instructions": "Does `item.text` explicitly carry out, preserve, or plan for the directive given in `anchor.statement`? Look for the directive's specific content being present in the item: the same starting point, the same method, the same constraint, or an explicit step that does what the statement asks. A generic mention of the topic without the directive's specific content does not count. Judge presence of the directive only, not whether the item conflicts with it.",
        "criteria": {
          "yes": "The item text contains the directive's specific content, in its own words or the user's, as a step, constraint, or stated starting point.",
          "no": "The item text is silent about the directive: it neither includes the directive's specific content nor references it.",
          "unclear": "The item text alludes to the directive's topic but it is not possible to tell whether the directive itself is included."
        }
      },
      "contradicted": {
        "type": "choice",
        "instructions": "Does `item.text` propose doing something that `anchor.statement` rules out, or propose a different approach than the one `anchor.statement` requires? Examples of contradiction: the anchor says to start from an existing implementation and the item plans a fresh implementation; the anchor forbids a dependency and the item adds it; the anchor fixes an order of work and the item reverses it. Silence about the directive is not a contradiction. Judge conflict only.",
        "criteria": {
          "yes": "The item text states a plan, approach, or decision that cannot be reconciled with the directive in the anchor statement.",
          "no": "Nothing in the item text conflicts with the directive, whether or not the directive is mentioned.",
          "unclear": "The item text is ambiguous enough that it might or might not conflict with the directive."
        }
      }
    },
    "candidate_minimum": 3,
    "repeat_count": 2,
    "tournament_protocol_version": "plan-anchor/question-tournament/2.0.0",
    "production_confidence_threshold": 0.6,
    "confidence_reference_threshold": 0.8,
    "tuning_history": {
      "schema": "plan-anchor/tuning-history/2",
      "history_id": "sc-runtime-phase-c-anc-001-planning-authority-v4",
      "events": []
    },
    "diagnostic_plan": null
  }
}
```
