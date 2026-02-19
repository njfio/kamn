# Issue #3962 Spec

- Title: Subtask: add CI dry-run governance checks and runbook marker parity for deployment hardening lane
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-6-key-custody-multi-signer-controls-and-deployment-hardening/index.md`

## Problem Statement
Deployment preflight policy/checker coverage exists, but issue-local closure contracts do not pin deterministic CI dry-run governance markers and runbook-parity taxonomy markers in a Rust docs-contract lane tied to `#3962`.

## Acceptance Criteria
- AC-1: CI dry-run governance markers for deployment preflight policy checks are declared in `docs/ci/strategy.md` and fail closed on drift.
- AC-2: Runbook marker parity taxonomy/version/reason-code markers are declared deterministically and validated by Rust docs-contract tests.
- AC-3: Production next-steps docs include explicit closure chain/guard-command markers for `#3962`.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Scope
In scope:
- `crates/kamn-core/tests/deployment_hardening_ci_dry_run_contract.rs`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `specs/3962/spec.md`
- `specs/3962/plan.md`
- `specs/3962/tasks.md`

Out of scope:
- New shell/python lane implementations.
- CI workflow topology changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | CI dry-run governance marker list | required markers are present and deterministic |
| C-02 | AC-1/AC-2 | Functional | CI strategy + policy checker marker assertions | missing governance/parity marker fails closed |
| C-03 | AC-2/AC-3 | Integration | strategy + next-steps chain/guard marker parity | chain and guard markers remain aligned |
| C-04 | AC-4 | Regression | targeted Rust docs-contract suite | suite fails on drift and passes on aligned markers |

## Test Mapping
- `cargo test -p kamn-core --test deployment_hardening_ci_dry_run_contract -- --nocapture`

## Success Metrics
- `#3962` closure is backed by deterministic Rust docs-contract checks with zero shell LOC increase.
