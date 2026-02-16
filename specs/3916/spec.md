# Issue #3916 Spec

- Title: `Subtask: add fail-closed policy checks for fallback signer keys and lifecycle markers`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Fallback signer key violations and incomplete lifecycle marker sets must be rejected deterministically to preserve secure-profile posture.

## Scope
In:
- Add policy checker helper checks for forbidden fallback reason code.
- Add required lifecycle-marker completeness checks.

Out:
- Decode-path implementation changes.

## Acceptance Criteria
- AC-1: fallback reason code `fallback_signer_secret_present_violation` is rejected.
- AC-2: missing required lifecycle markers are rejected.
- AC-3: complete marker sets without forbidden reason codes are accepted.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract policy_checker_rejects_fallback_secret_violation_reason_code -- --exact --nocapture` | checker rejects forbidden reason code |
| C-02 | AC-2 | Functional | `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract policy_checker_rejects_missing_required_lifecycle_markers -- --exact --nocapture` | checker rejects incomplete marker set |
| C-03 | AC-3 | Functional | `cargo test -p kamn-node --test signer_secret_lifecycle_policy_contract policy_checker_accepts_complete_marker_set_without_fallback_violation -- --exact --nocapture` | checker accepts complete non-violating markers |

## Test Mapping
- `crates/kamn-node/tests/signer_secret_lifecycle_policy_contract.rs`

## Success Metrics
- fallback and marker-completeness policy checks are deterministic and fail-closed.
