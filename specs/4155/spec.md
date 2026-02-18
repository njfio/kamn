# Issue #4155 Spec

- Title: Subtask: implement rehearsal lineage verifier and deterministic promotion gate reason-code mapping
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-19-live-deployment-rehearsal-and-rollback-governance-hardening/index.md

## Problem Statement
Promotion gating requires deterministic upgrade-lineage and promotion-gate reason-code projections when rehearsal lineage is incomplete or tampered.

## Acceptance Criteria
- AC-1: Upgrade-lineage checker returns deterministic reason-code CSV/value markers for rollback lineage missing failures.
- AC-2: Upgrade-lineage checker returns deterministic reason-code CSV/value markers for recovery lineage missing failures.
- AC-3: Runbook/checklist guidance documents deterministic promotion-gate lineage reason mapping markers for fail-closed review decisions.

## Scope
In scope:
- Extend lineage regression tests in `scripts/deploy/test_generate_gonogo_evidence_bundle.sh` to assert explicit upgrade-lineage and promotion-gate reason mapping outputs on rollback/recovery lineage failures.
- Update `docs/foundation/release-gonogo-checklist.md` with explicit promotion-gate mapping markers for rollback/recovery lineage failures.
- Add lifecycle artifacts for issue `#4155`.

Out of scope:
- Changes to milestone bundle schema versions.
- Changes to live-node validation generator structure outside deterministic reason-code surface checks.

## Shell-Surface Impact Estimates
- shell_loc_delta_estimate: 24
- rust_loc_delta_estimate: 0
- shell_to_rust_ratio_delta_estimate: 0.0002
- shell_surface_mitigation_issue: #4148

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Regression | Milestone bundle with rollback lineage artifact path removed | `check_upgrade_rehearsal_lineage_policy.py` reports `upgrade_lineage_reason_codes_csv=milestone_review_live_node_validation_rollback_lineage_missing` and matching promotion-gate reason CSV/value markers |
| C-02 | AC-2 | Regression | Milestone bundle with recovery lineage artifact path removed | `check_upgrade_rehearsal_lineage_policy.py` reports `upgrade_lineage_reason_codes_csv=milestone_review_live_node_validation_recovery_lineage_missing` and matching promotion-gate reason CSV/value markers |
| C-03 | AC-3 | Functional | Release checklist promotion-gate section | Explicit deterministic mapping guidance includes rollback/recovery lineage reason-code markers |

## Test Mapping
- `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh`

## Success Metrics
- `#4155` closes with deterministic promotion-gate reason mapping assertions for rollback/recovery lineage failures and aligned release checklist documentation.
