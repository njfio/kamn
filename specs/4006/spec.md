# Issue #4006 Spec

- Title: Subtask: implement ci dry-run capacity governance checker and go-no-go marker parity tests
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement
We need a low-cost, fail-closed CI governance check that verifies capacity evidence remains deterministic in dry-run mode and that release go/no-go markers do not drift. Current CI includes component-level contracts, but this subtask needs a composed capacity dry-run governance gate with deterministic reason taxonomy and selector/workflow parity checks.

## Acceptance Criteria
- AC-1: Add a fail-closed capacity CI dry-run governance checker that validates performance smoke evidence and go/no-go gate evidence against versioned thresholds/contracts.
- AC-2: Checker must emit deterministic GO/NO-GO status with stable reason taxonomy + reason-codes CSV/value markers.
- AC-3: CI fast-mode selector and workflow exclusion contracts are enforced (required checker coverage path present; forbidden run-mode invocation absent).
- AC-4: Documentation markers in `docs/ci/strategy.md` stay in parity with checker contracts and remediation taxonomy.
- AC-5: Unit, Functional, Integration, Regression, and Performance contract tests are present and pass.

## Scope
In scope:
- New Python checker for capacity CI dry-run governance.
- Threshold fixture for checker contracts/taxonomy.
- Contract tests in `crates/kamn-core/tests`.
- CI tool-chain wiring and strategy doc marker updates.

Out of scope:
- New run-mode load execution in CI fast-gate.
- Changes to runtime go/no-go lane behavior or release policy semantics.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | baseline performance + go/no-go dry-run reports | checker returns `status=pass`, `final_decision=GO` |
| C-02 | AC-2 | Functional | tampered report contract field | checker fails closed with deterministic report-drift reason |
| C-03 | AC-3 | Integration | workflow/selector fixture with leaked run-mode command | checker fails closed with selector/workflow drift reason |
| C-04 | AC-4 | Regression | docs fixture missing remediation marker | checker fails closed with docs remediation drift reason |
| C-05 | AC-5 | Performance | baseline checker run | checker completes within bounded runtime budget |

## Test Mapping
- `cargo test -p kamn-core --test capacity_ci_dry_run_governance_contract -- --nocapture`

## Success Metrics
- Capacity governance checker emits stable schema/taxonomy markers and deterministic reason ordering.
- CI fast-mode includes the new contract test while preserving run-mode exclusion in fast-gate governance.
