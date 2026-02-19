# Issue #3961 Spec

- Title: Subtask: implement local-heavy deployment hardening lane for secretless startup and rotation drill artifacts
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: `specs/milestones/r27-6-key-custody-multi-signer-controls-and-deployment-hardening/index.md`

## Problem Statement
Deployment preflight lane scripts already exist, but issue-local contract evidence does not explicitly pin local-heavy secretless startup/restart/rotation artifact markers in Rust tests and milestone closure docs.

## Acceptance Criteria
- AC-1: Deployment preflight lane artifact contract test verifies deterministic secretless startup/restart/rotation markers.
- AC-2: Lane reason taxonomy markers and run-mode boundary markers are fail-closed via Rust contract tests.
- AC-3: Production next-steps plan includes deterministic closure markers and guard commands for this lane.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Scope
In scope:
- `crates/kamn-core/tests/deployment_hardening_lane_contract.rs`
- `docs/plans/2026-02-14-production-service-next-steps.md`
- `specs/3961/spec.md`
- `specs/3961/plan.md`
- `specs/3961/tasks.md`

Out of scope:
- New shell/python lane implementations.
- CI-fast run-mode execution changes.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | deployment lane marker extraction helpers | required markers parse deterministically |
| C-02 | AC-1/AC-2 | Functional | docs + script marker assertions | missing marker fails closed |
| C-03 | AC-2 | Integration | run-mode boundary and reason-code parity between docs and lane impl | dry-run/run markers and reason taxonomy remain aligned |
| C-04 | AC-3/AC-4 | Regression | closure markers + guard command docs-contract test | drift fails closed and suite passes |

## Test Mapping
- `cargo test -p kamn-core --test deployment_hardening_lane_contract -- --nocapture`

## Success Metrics
- Local-heavy deployment hardening lane artifact markers remain contract-pinned without adding shell LOC.
