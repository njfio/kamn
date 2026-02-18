# Issue #4478 Spec

- Title: `Task: implement anti-flake classifier and deterministic rerun-policy checker contracts`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-41-tls-governance-completion-and-anti-flake-merge-gate-reliability-contracts/index.md`
- Parent: `#4475`

## Problem Statement
Merge reliability policy requires deterministic anti-flake classification and rerun-policy contract checks so bypasses and drift fail closed with stable reason outputs.

## Scope
In:
- Deterministic anti-flake reason taxonomy and normalized policy output markers.
- Rerun-policy checks for CI fast/deep workflow bounded retry invariants.
- Anti-flake strategy docs marker updates and parity checks.

Out:
- CI vendor-specific retry features.
- Workflow architecture redesign.

## Acceptance Criteria
- AC-1: flake signatures are classified deterministically with stable reason taxonomy markers.
- AC-2: rerun-policy violations fail closed with deterministic reason codes.
- AC-3: anti-flake policy checker emits normalized merge-evidence output markers.
- AC-4: regression tests preserve anti-flake classifier and rerun-policy behavior.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `bash scripts/ci/test_check_anti_flake_policy.sh` | deterministic reason taxonomy markers emitted on pass/fail paths |
| C-02 | AC-2 | Functional | `bash scripts/ci/test_check_anti_flake_policy.sh` | rerun-policy violations fail closed with stable reason codes |
| C-03 | AC-2 | Integration | `bash scripts/ci/test_workflow_retry_policy.sh` | workflow retry boundaries stay deterministic across fast/deep lanes |
| C-04 | AC-4 | Regression | `bash scripts/ci/test_check_anti_flake_policy.sh` | registry/rerun-policy bypass and expected-decision drift remain fail-closed |
| C-05 | AC-1 | Performance | `bash scripts/ci/test_check_anti_flake_policy.sh` | anti-flake checker remains bounded and lightweight |
| C-06 | AC-3 | Docs | `cargo test -p kamn-core --test ci_strategy_docs doc_contains_anti_flake_rerun_policy_reason_taxonomy_markers -- --exact` | CI strategy includes deterministic anti-flake/rerun-policy markers |

## Test Mapping
- `scripts/ci/check_anti_flake_policy.sh`
- `scripts/ci/test_check_anti_flake_policy.sh`
- `scripts/ci/test_workflow_retry_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Success Metrics
- Anti-flake outputs include deterministic reason taxonomy and normalized reason CSV/value markers.
- Rerun-policy drift across CI workflows is detected fail closed with stable reason codes.
- Docs parity tests block anti-flake/rerun-policy marker drift.
