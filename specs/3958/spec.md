# Issue #3958 Spec

- Title: Subtask: add quorum drift and signer-disagreement policy checker with go-no-go markers
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-6-key-custody-multi-signer-controls-and-deployment-hardening/index.md`

## Problem Statement
The real-node runtime profile policy checker emits signature decision reason codes, but it does not emit explicit quorum drift and signer-disagreement go/no-go marker fields that downstream governance tooling can consume directly.

## Acceptance Criteria
- AC-1: `scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py` emits deterministic quorum drift and signer-disagreement go/no-go marker fields in JSON and stdout key-value output.
- AC-2: Drift/disagreement marker fields map deterministically to signature-decision reason subsets and fail closed on NO-GO scenarios.
- AC-3: `docs/ci/strategy.md` documents the new go/no-go marker contracts and taxonomy markers for the real-node profile policy checker.
- AC-4: Existing policy checker contract tests and docs contract tests cover the new markers and pass.

## Scope
In scope:
- `scripts/kolme/check_local_kamn_live_runtime_real_node_profile_policy.py`
- `scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/3958/spec.md`
- `specs/3958/plan.md`
- `specs/3958/tasks.md`

Out of scope:
- New shell/python lane executables.
- Changes to runtime signer preflight logic already covered by #3957.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Functional | GO fixture report in checker contract test | quorum drift/disagreement statuses are `verified` and go/no-go markers are `GO` |
| C-02 | AC-1/AC-2 | Regression | quorum linkage drift fixture | quorum drift status is fail-closed with go/no-go marker `NO-GO` |
| C-03 | AC-1/AC-2 | Regression | attestation quorum shortfall fixture | signer-disagreement status is fail-closed with go/no-go marker `NO-GO` |
| C-04 | AC-3 | Functional | CI strategy docs section | docs declare quorum drift/disagreement go/no-go marker taxonomy and expected values |
| C-05 | AC-4 | Integration | shell policy checker contract test + docs contract test | all marker assertions pass with deterministic output |

## Test Mapping
- `bash scripts/kolme/test_check_local_kamn_live_runtime_real_node_profile_policy.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_signer_quorum_go_no_go_policy_markers`

## Success Metrics
- Checker output includes explicit quorum drift/disagreement go/no-go markers for both pass and fail fixtures.
- No new shell/python executables added.
- Shell LOC delta stays bounded and documented in closure.
