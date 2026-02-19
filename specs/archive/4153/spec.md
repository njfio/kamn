# Issue #4153 Spec

- Title: Subtask: implement rollback simulation lane contracts and policy checker parity validation
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-19-live-deployment-rehearsal-and-rollback-governance-hardening/index.md

## Problem Statement
Rollback simulation governance needs explicit contract-lane parity checks so lane outputs and policy checker decisions remain deterministic under trigger mismatch and taxonomy drift scenarios.

## Acceptance Criteria
- AC-1: Governance rollback contract lane validates deterministic GO and NO-GO policy decisions for rollback simulation fixtures.
- AC-2: Contract lane enforces rollback trigger mismatch and reason-taxonomy drift parity failures with deterministic markers.
- AC-3: CI strategy documentation captures rollback lane boundary and CI exclusion policy markers used by the contract lane.

## Scope
In scope:
- `scripts/governance/governance_lifecycle_rollback_contract_lane_contract.py` parity validation enhancements.
- `scripts/governance/test_run_governance_lifecycle_rollback_contract_lane.sh` regression coverage for new parity markers.
- `docs/ci/strategy.md` rollback lane boundary marker synchronization.
- Lifecycle artifacts for `#4153`.

Out of scope:
- Governance lifecycle rollback lane architecture changes.
- New rollback orchestration behavior.

## Shell-Surface Impact Estimates
- shell_loc_delta_estimate: 100
- rust_loc_delta_estimate: 0
- shell_to_rust_ratio_delta_estimate: 0.0000
- shell_surface_mitigation_issue: None

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | Contract lane GO/NO-GO fixture execution | Deterministic lane-policy parity decisions and markers are emitted |
| C-02 | AC-2 | Regression | Tampered rollback trigger projection fixture | Contract lane rejects mismatch via deterministic policy checker failure marker |
| C-03 | AC-2 | Regression | Tampered reason taxonomy CSV fixture | Contract lane rejects taxonomy drift via deterministic mismatch marker |
| C-04 | AC-3 | Conformance | CI strategy rollback section markers | Docs stay synchronized with rollback parity marker contract |

## Test Mapping
- `bash scripts/governance/test_run_governance_lifecycle_rollback_contract_lane.sh`

## Success Metrics
- `#4153` closes with deterministic rollback lane-policy parity checks and synchronized CI strategy markers.
