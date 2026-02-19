# Issue #4154 Spec

- Title: Subtask: add red tests for rehearsal bundle linked-artifact lineage completeness and tamper rejection
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-19-live-deployment-rehearsal-and-rollback-governance-hardening/index.md

## Problem Statement
Rehearsal-to-promotion aggregate evidence needs deterministic failing tests when linked-artifact lineage is incomplete or milestone lineage markers are tampered.

## Acceptance Criteria
- AC-1: Tests fail when rollback lineage link coverage is incomplete in live-node validation artifacts.
- AC-2: Tests fail when recovery lineage link coverage is incomplete in live-node validation artifacts.
- AC-3: Tests fail closed when linked-artifact lineage contract markers in the milestone bundle are tampered.

## Scope
In scope:
- Add red/regression lineage completeness and tamper-rejection tests in `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`.
- Add lifecycle artifacts for issue `#4154`.

Out of scope:
- Changes to lineage checker implementation logic.
- New lane/checker feature surfaces.

## Shell-Surface Impact Estimates
- shell_loc_delta_estimate: 120
- rust_loc_delta_estimate: 0
- shell_to_rust_ratio_delta_estimate: 0.0000
- shell_surface_mitigation_issue: None

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | Milestone bundle generated from live-node summary with rollback lineage link removed from `artifact_paths` | Deterministic NO-GO with `milestone_review_live_node_validation_rollback_lineage_missing` |
| C-02 | AC-2 | Regression | Milestone bundle generated from live-node summary with recovery lineage link removed from `artifact_paths` | Deterministic NO-GO with `milestone_review_live_node_validation_recovery_lineage_missing` |
| C-03 | AC-3 | Regression | Tampered `milestone_review_bundle.contracts.linked_artifact_lineage_required=false` | Policy checker and lineage checker reject with deterministic lineage mismatch |

## Test Mapping
- `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`

## Success Metrics
- `#4154` closes with deterministic linked-artifact lineage completeness and tamper rejection tests added and passing.
