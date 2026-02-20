# Issue #4005 Spec

- Title: Subtask: add load-lane policy checker for capacity thresholds and fail-closed reason taxonomy
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement

Capacity/governance load-lane policy checks must remain fail-closed with deterministic reason taxonomy.
The checker and its tests already enforce thresholds and marker parity, but the observability baseline docs do not currently pin the capacity dry-run taxonomy markers required for operator-facing drift detection.

## Scope

In scope:
- Document capacity dry-run threshold reason taxonomy markers in `docs/foundation/observability-slo-dashboards.md`.
- Add fail-closed docs contract tests that assert the documented taxonomy markers and regression guard text.
- Verify existing checker test coverage still provides unit/functional/integration/regression/performance coverage for threshold and marker enforcement.

Out of scope:
- Reworking checker control flow or release decision orchestration.
- New CI workflow behavior changes.

## Acceptance Criteria

- AC-1: Capacity governance checker remains fail-closed for threshold/marker violations.
- AC-2: Capacity threshold reason taxonomy is deterministic and documented in observability baseline docs.
- AC-3: Unit, Functional, Integration, and Regression coverage for the checker behavior is present and passing.
- AC-4: Performance guard for checker runtime remains enforced and passing.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | tampered go/no-go marker payload | checker returns fail/NO-GO with deterministic reason code |
| C-02 | AC-2 | Unit | observability doc content assertions for capacity taxonomy markers | docs contract test passes only when markers are present |
| C-03 | AC-3 | Integration | workflow/selector drift fixture in governance contract suite | checker fails closed with deterministic drift reason |
| C-04 | AC-2 | Regression | missing fail-closed taxonomy regression marker text in docs | docs contract test fails closed |
| C-05 | AC-4 | Performance | baseline checker invocation | checker runtime remains within configured budget |

## Test Mapping

- `cargo test -p kamn-core --test capacity_ci_dry_run_governance_contract -- --nocapture`
- `cargo test -p kamn-core --test observability_stack_docs -- --nocapture`

## Success Metrics / Observable Signals

- Observability baseline docs include stable `capacity_ci_dry_run_*` reason taxonomy markers.
- Docs contract tests fail closed when taxonomy or regression marker text drifts.
- Capacity governance contract suite remains green across unit/functional/integration/regression/performance cases.
