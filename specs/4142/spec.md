# Issue #4142 Spec

- Title: Subtask: update validation-depth docs and drift-contract checks for property fuzz concurrency closure
- Status: Implemented
- Type: subtask
- Priority: P2
- Milestone: specs/milestones/r27-18-advanced-validation-depth-and-deterministic-assurance-hardening/index.md

## Problem Statement
Validation-depth docs and drift-check contracts must stay synchronized for property/fuzz/concurrency closure evidence; mismatched marker taxonomy would invalidate governance proof.

## Acceptance Criteria
- AC-1: Docs and checkers expose consistent marker taxonomy/command surface for invariant-fuzz-concurrency closure.
- AC-2: Drift-contract tests fail closed on marker mismatch.
- AC-3: Closure evidence keeps CI-smoke boundaries explicit and local-heavy paths opt-in.

## Scope
In scope:
- `docs/ci/strategy.md`
- `docs/testing/invariant-and-fuzz-strategy.md`
- `docs/foundation/{runtime-network.md,invariants.md,performance-target-benchmarking.md}`
- Docs-contract + runtime script tests that enforce marker parity
- `specs/4142/{spec.md,plan.md,tasks.md}`

Out of scope:
- New deep-lane implementations.
- CI workflow topology changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | `cargo test -p kamn-core --test ci_strategy_docs` | Invariant-fuzz-concurrency marker taxonomy/commands present |
| C-02 | AC-2 | Regression | `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh` | Docs drift on required markers fails closed |
| C-03 | AC-3 | Functional | `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh` | CI-smoke boundary and local-heavy mode markers enforced |

## Test Mapping
- `cargo test -p kamn-core --test ci_strategy_docs -- doc_contains_invariant_fuzz_concurrency_ci_smoke_boundary_contract_markers`
- `bash scripts/runtime/test_run_invariant_fuzz_concurrency_contract_lane.sh`
- `bash scripts/runtime/test_check_invariant_fuzz_concurrency_policy.sh`

## Success Metrics
- Marker taxonomy parity is contract-tested across docs and scripts.
- Drift contracts remain fail-closed.
- Shell surface growth for this closure pass is `0`.
