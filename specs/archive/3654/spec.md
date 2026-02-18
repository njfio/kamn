# Issue #3654 Spec

- Title: `Subtask: extract signer_policy with deterministic quorum and profile checks`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Signer policy validation and reason taxonomy needed dedicated module ownership to reduce coupling and keep fail-closed behavior deterministic.

## Scope
In:
- Ensure signer policy ownership for profile normalization, quorum linkage, and deterministic reason taxonomy.
- Preserve runtime policy decisions and fail-closed semantics.
- Add policy drift contracts.

Out:
- Adapter-level crypto implementation changes.

## Acceptance Criteria
- AC-1: signer policy module owns profile/quorum checks.
- AC-2: policy reason taxonomy remains deterministic.
- AC-3: policy behavior remains stable in live/signer lanes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Functional | `cargo test -p kamn-node --test signer_policy_reason_taxonomy_contract` | taxonomy contract passes |
| C-02 | AC-1 | Functional | `bash scripts/signer/test_run_signer_policy_contract_lane.sh` | signer policy contract lane passes |
| C-03 | AC-3 | Integration | `bash scripts/signer/test_run_signer_emulator_contract_lane.sh` | signer emulator lane preserves policy behavior |
| C-04 | AC-2/AC-3 | Regression | `bash scripts/kolme/test_check_fallback_signer_marker_matrix_policy.sh` | fallback marker taxonomy checks pass |

## Test Mapping
- `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- `scripts/signer/test_run_signer_policy_contract_lane.sh`
- `scripts/signer/test_run_signer_emulator_contract_lane.sh`
- `scripts/kolme/test_check_fallback_signer_marker_matrix_policy.sh`

## Success Metrics
- Signer policy ownership and deterministic taxonomy remain enforced.
