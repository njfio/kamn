# Issue #4479 Spec

- Title: `Task: enforce merge-gate reliability evidence convergence and CI smoke/local-heavy boundary governance`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-41-tls-governance-completion-and-anti-flake-merge-gate-reliability-contracts/index.md`
- Parent: `#4475`

## Problem Statement
Merge-gate anti-flake enforcement needs deterministic reliability reason taxonomy and fail-closed boundary checks so CI smoke remains bounded and local-heavy lanes remain explicit opt-in.

## Scope
In:
- Deterministic merge-gate reliability reason taxonomy and normalized marker outputs in anti-flake policy checks.
- Fail-closed CI smoke/local-heavy boundary checks against fast-gate workflow markers.
- Red/green regression coverage and CI strategy docs parity updates.

Out:
- Workflow architecture rewrites.
- Vendor-specific CI policy extensions.

## Acceptance Criteria
- AC-1: Reliability evidence mismatch and convergence gaps fail closed with deterministic reason codes.
- AC-2: CI smoke/local-heavy boundary violations fail closed with deterministic reason codes.
- AC-3: Merge-gate checker outputs auditable normalized markers for reliability reason taxonomy.
- AC-4: Regression and docs parity tests preserve deterministic merge-gate reliability behavior.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/ci/test_check_anti_flake_policy.sh` | fails closed on evidence convergence gaps with deterministic reason codes |
| C-02 | AC-2 | Integration | `bash scripts/ci/test_check_anti_flake_policy.sh` | fails closed when CI smoke/local-heavy workflow boundary markers drift |
| C-03 | AC-3 | Functional | `bash scripts/ci/test_check_anti_flake_policy.sh` | emits taxonomy version and normalized reason markers (`csv/value/class`) |
| C-04 | AC-4 | Regression | `bash scripts/ci/test_anti_flake_merge_gate_policy.sh` and `bash scripts/ci/test_ci_strategy_contract.sh` | docs/workflow parity remains synchronized |
| C-05 | AC-4 | Docs | `cargo test -p kamn-core --test ci_strategy_docs doc_contains_merge_gate_reliability_ci_smoke_local_heavy_boundary_markers -- --exact` | docs include deterministic merge-gate reliability marker surface |
| C-06 | AC-2 | Performance | `bash scripts/ci/test_check_anti_flake_policy.sh` | smoke/local-heavy boundary checks remain bounded and deterministic |

## Test Mapping
- `scripts/ci/check_anti_flake_policy.sh`
- `scripts/ci/test_check_anti_flake_policy.sh`
- `scripts/ci/test_anti_flake_merge_gate_policy.sh`
- `scripts/ci/test_ci_strategy_contract.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Success Metrics
- Merge-gate checker emits deterministic reliability taxonomy/version markers and normalized reason marker set.
- Convergence and boundary drift paths fail closed with stable reason codes.
- Docs and contract tests block drift for merge-gate reliability boundary markers.
