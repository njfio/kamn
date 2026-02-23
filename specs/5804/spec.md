# Spec: Issue #5804 - Activate R55 with New kamn-core Live Probe Matrix Module

- Issue: #5804
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Problem Statement
R54 set an explicit activation target for R55: deliver at least one new `kamn-core` module. Current module inventory is stagnant. We need a real core surface that models live probe matrix outcomes with deterministic, fail-closed semantics.

## Acceptance Criteria
- AC-1: Add a new public `kamn-core` module and export it through `crates/kamn-core/src/lib.rs`.
- AC-2: Module provides typed mode/scenario outcome structures with validation that fails closed on duplicates/invalid inputs.
- AC-3: Module provides deterministic aggregate status helpers for per-mode and overall matrix evaluation.
- AC-4: Unit + contract tests cover happy, error, and edge behavior for new APIs.
- AC-5: Lifecycle artifacts and milestone metadata are finalized for #5804.

## Scope
In scope:
- `crates/kamn-core/src/live_probe_matrix.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/live_probe_matrix_contract.rs`
- `specs/5804/spec.md`
- `specs/5804/plan.md`
- `specs/5804/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

Out of scope:
- Harness runtime execution changes.
- CI/workflow changes.
- Protocol/wire-format changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Conformance | Build crate and inspect exports | New module is public and importable via `kamn_core`. |
| C-02 | AC-2 | Unit | Duplicate/invalid matrix entries | Validation rejects bad input with deterministic error variants. |
| C-03 | AC-3 | Functional | Mixed PASS/FAIL/SKIP matrix | Aggregate helpers return deterministic per-mode and overall outcomes. |
| C-04 | AC-4 | Regression | `live_probe_matrix_contract` suite | Happy/error/edge coverage remains green. |
| C-05 | AC-5 | Conformance | Lifecycle + milestone files | #5804 artifacts and milestone delivery slice are finalized. |

## Test Mapping
- `cargo test -p kamn-core --test live_probe_matrix_contract -- --nocapture`
- `cargo test -p kamn-core live_probe_matrix -- --nocapture`
- `cargo fmt --check`
- `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract -- --nocapture`
- `cargo test -p kamn-core --test review_r53_docs_contract -- --nocapture`

## Success Metrics
- `kamn-core` module count increases by one.
- New API has deterministic validation/aggregation behavior and regression coverage.
- Non-regression docs-contract gates remain green.
