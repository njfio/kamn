# Issue #4018 Spec

- Title: Subtask: add corruption-recovery policy checker with runbook-docs marker parity tests
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Problem Statement

Issue `#4017` introduced a deterministic sqlite crash-restart local-heavy runner, but there is no
dedicated policy checker that fail-closes on corruption/recovery marker drift while also enforcing
runbook/docs parity for operator-facing governance.

## Scope

In scope:
- Add a crash-restart policy checker for the `#4017` runner artifact.
- Enforce fail-closed validation for corruption/recovery/profile markers.
- Enforce runbook + docs marker parity checks.
- Add unit/functional/integration/regression/performance test coverage.
- Update `docs/ci/strategy.md` and runbook parity markers used by the checker.

Out of scope:
- Running heavy run-mode drills in CI fast-gate.
- Platform-level runbook migrations.

## Shell-Surface Estimates

- shell_loc_delta_estimate: 140
- rust_loc_delta_estimate: 360
- shell_to_rust_ratio_delta_estimate: -0.0003
- shell_surface_mitigation_issue: None

## Acceptance Criteria

- AC-1: Policy checker fails closed on corruption/recovery marker drift in runner artifacts.
- AC-2: Policy checker enforces deterministic reason taxonomy + reason-code projection.
- AC-3: Runbook/docs marker parity is enforced and drift fails closed.
- AC-4: Required documentation updates are present in `docs/ci/strategy.md`.
- AC-5: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Unit | valid dry-run artifact + checker | `status=pass`, `final_decision=GO`, deterministic taxonomy markers |
| C-02 | AC-1 | Functional | tampered runner marker payload | checker fails closed with deterministic marker drift reason |
| C-03 | AC-3 | Integration | runner + checker + strategy/runbook docs | parity status markers verified, reason code `none` |
| C-04 | AC-3 | Regression | drifted runbook marker | checker fails closed with deterministic runbook parity reason |
| C-05 | AC-3/AC-4 | Regression | drifted strategy marker | checker fails closed with deterministic strategy parity reason |
| C-06 | AC-5 | Performance | checker execution baseline | completes within bounded local budget |

## Test Mapping

- `cargo test -p kamn-core --test sqlite_crash_restart_local_heavy_policy_contract -- --nocapture`
- `bash scripts/runtime/test_check_sqlite_crash_restart_local_heavy_policy.sh`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_sqlite_crash_restart_local_heavy_policy_checker_contract -- --exact`
- `cargo test -p kamn-core --test kolme_devnet_ops_docs deploy_compat_contains_sqlite_crash_restart_runbook_policy_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics / Observable Signals

- Crash-restart policy checker provides deterministic fail-closed governance on artifact drift.
- Runbook and strategy markers remain synchronized through contract tests.
- Checker remains fast enough for low-cost CI smoke usage.
