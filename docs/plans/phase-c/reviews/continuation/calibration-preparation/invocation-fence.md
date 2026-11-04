```json
{
  "schema_version": "1.0.0",
  "runtime": {
    "agent": "anchor-tuning",
    "harness": "codex",
    "model": "sol",
    "path": ".codex/agents/anchor-tuning.md",
    "sha256": "60cf04b170a95c00bd2be5bf46107985af6685da8d69b6cc4898fccd1e27d798",
    "version": "0.8.7"
  },
  "operation": "testability",
  "correlation_id": "sc-runtime-testability-continuation-087-r1",
  "params": {
    "schema_version": "2.0.0",
    "operation": "testability",
    "gate": "testability",
    "template": {
      "id": "anchor-tuning-testability-request",
      "version": "0.4.1",
      "sha256": "63a243019416001d3aa034523e510b0817e9af21058592961e413b13ff10b909"
    },
    "correlation_id": "sc-runtime-testability-continuation-087-r1",
    "runtime": {
      "workspace": "/Users/randlee/github/sc-runtime-worktrees/plan/phase-c",
      "script": "/Users/randlee/github/sc-runtime-worktrees/plan/phase-c/.codex/skills/plan-anchoring/scripts/jev_judge.py",
      "version": "0.8.7",
      "sha256": "be99ae73699f25d5385e3611461eb5d874cd9ce72d7c451995869f09fa3012de",
      "contract_script": "/Users/randlee/github/sc-runtime-worktrees/plan/phase-c/.codex/skills/plan-anchoring/scripts/anchor_tuning_contract.py",
      "contract_sha256": "769cd7f5729f2662fc7018a9a4b3356bc203c0741c6a86adb15c0a92004bfdc0",
      "agent": "anchor-tuning",
      "harness": "codex",
      "model": "sol"
    },
    "authority": {
      "intent": {
        "id": "sc-runtime-testability-calibration-continuation",
        "version": 1,
        "content": "Independent append-only continuation of the testability-rubric calibration. Reopen only the faithful and single_constraint roles; preserve observable, scoped, and self_contained questions byte-for-byte and run the complete five-role regression. The current seal is 96d8f74cf022d7515aaa0d4a8f4e103b9be8c7b776bf99794d9926ef13d8fdb4. Two independently validated triage decisions reopened these roles; their active-anchor cases are excluded from this calibration oracle. The only supplied oracle is the unchanged shipped frozen 17-case plan-independent corpus. Relevant defect categories are preservation of authorized meaning versus invention of policy or precision, and a single predicate across several output surfaces versus genuinely independent duties; existing corpus examples cover these distinctions. Do not supply or optimize for an active plan or accepted-anchor result. Do not relabel, remove, or replace any original corpus case. If approved-clarification distinctions require additional generic calibration examples, independently propose and freeze a separate supplement with authority-derived labels before any response; retain the original corpus and digest unchanged and identify any missing authority. No planner question wording or actual-anchor expectation is supplied. Preserve all prior candidates, raw responses, decisions, and seals; continuation artifacts are additive. Current production rubric input may lack approved clarification context. Report that capability limit if faithful cannot be calibrated truthfully on the available input; do not invent context or claim production support for absent fields. Production threshold remains 0.6, confidence reference 0.8, and byte-identical repeat count at least two; low confidence remains a failure under production semantics.",
        "sha256": "b7d20ab0681a3b1abe2dc2aa48b43eed0992a23f17f4d3ea6a04abeddb234f1d"
      },
      "anchors": {
        "sha256": "4f53cda18c2baa0c0354bb5f9a3ecbe5ed12ab4d8e11ba873c2f11161202b945",
        "records": []
      }
    },
    "calibration_corpus": {
      "schema": "plan-anchor/testability-calibration/2",
      "version": 2,
      "path": "/Users/randlee/github/sc-runtime-worktrees/plan/phase-c/.codex/skills/plan-anchoring/assets/fixtures/testability-calibration.v2.json",
      "sha256": "ce5f4efcfee74ecc767bb8c140bf749ef2e33c779f82397d5c42d96698a7b1bc",
      "case_count": 17,
      "frozen_before_live": true
    },
    "current_questions": {
      "faithful": {
        "criteria": {
          "no": "The requirement commits to an additional policy, implementation selection, strength, trigger or result.",
          "unclear": "Meaning preservation cannot be decided from the quote and statement.",
          "yes": "The user duty is preserved with ordinary language, channel or representation normalization only."
        },
        "instructions": "Does anchor.quote substantively authorize the duty in anchor.statement? Read informal user language as a requirement. Allow one atomic duty from several quoted duties, technical output-channel formalization for printing or logging, and ordinary JSON container terminology where no container distinction was specified. Preserve vagueness rather than inventing precision. Reject added policy commitments, including selecting an encryption algorithm or strength absent from the quote.",
        "type": "choice"
      },
      "observable": {
        "criteria": {
          "no": "It supplies only a subjective quality, undefined shorthand, or an approval with no observable record.",
          "unclear": "Some evidence is named, but the observation needed to settle compliance remains ambiguous.",
          "yes": "It names evidence or behavior that can be inspected with a determinate outcome."
        },
        "instructions": "Does anchor.statement define a checkable behavior or artifact without relying on an undefined label or an approval that has no identified record? Judge only whether the stated outcome can be observed.",
        "type": "choice"
      },
      "scoped": {
        "criteria": {
          "no": "No work area or applicability set is identifiable.",
          "unclear": "An area is named but its applicability cannot be determined.",
          "yes": "A reviewer can identify the work area where this anchor applies, including an explicit all-items set."
        },
        "instructions": "Do anchor.statement and anchor.scope identify the governed activity, component, artifact, or explicit global set, independent of whether the requirement itself is vague or otherwise defective?",
        "type": "choice"
      },
      "self_contained": {
        "criteria": {
          "no": "At least one decisive term is an undefined quality or private label, or compliance depends on an approval with no identified record.",
          "unclear": "A possible outside-context dependency exists but the statement does not show whether it is decisive.",
          "yes": "None of those outside-context dependencies is needed; ordinary technical terms, concrete prohibitions, lists, and explicit input/action/outcome conditions are sufficient."
        },
        "instructions": "Is anchor.statement independently understandable and applicable without any of these disqualifiers: an undefined subjective quality (for example robust), private shorthand, or a human approval that has no identified recorded artifact? Ignore grammar, bundling, fidelity, and scope.",
        "type": "choice"
      },
      "single_constraint": {
        "criteria": {
          "no": "At least two independently enforceable duties are joined in the statement.",
          "unclear": "The statement leaves independence of its duties ambiguous.",
          "yes": "All details enforce the same invariant and would be evaluated together."
        },
        "instructions": "Is anchor.statement one invariant applied to a subject, trigger, and possibly several output surfaces? Do not count the number of nouns or listed artifacts; count independently enforceable duties.",
        "type": "choice"
      }
    },
    "candidate_minimum": 3,
    "repeat_count": 2,
    "tournament_protocol_version": "plan-anchor/question-tournament/2.0.0",
    "production_confidence_threshold": 0.6,
    "confidence_reference_threshold": 0.8
  }
}
```
