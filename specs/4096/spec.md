# Issue #4096 Spec

- Title: Subtask: implement ci dry-run overload governance checker and baseline threshold fixtures
- Status: Reviewed
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
Fast-gate needs a deterministic dry-run policy checker for daemon OS-signal overload evidence so threshold drift and selector regressions fail closed before release promotion.

## Acceptance Criteria
- AC-1: A CI dry-run overload governance checker validates report schema, decision markers, and threshold budget drift fail-closed.
- AC-2: Baseline threshold fixture(s) exist and are consumed by the checker.
- AC-3: Selector policy keeps local-heavy overload runner execution out of fast-gate by enforcing command-surface contract checks.
- AC-4: Unit/Functional/Integration/Regression tests pass for checker + fixture + selector composition.

## Scope
In scope:
- Add checker script under `scripts/ci/` for daemon overload dry-run report policy validation.
- Add baseline threshold fixture under `fixtures/ci/` used by the checker.
- Add shell regression tests for pass/fail policy behavior and selector guard checks.
- Wire checker regression test into `scripts/ci/test_ci_tools.sh`.
- Update `docs/ci/strategy.md` with checker command/fixture/marker contract references.
- Add/update Rust docs-contract assertions for new strategy markers.
- Add `specs/4096/{spec.md,plan.md,tasks.md}`.

Out of scope:
- Running local-heavy stress execution in fast-gate.
- Runtime daemon behavior changes.
- Workflow topology redesign.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 180
- rust_loc_delta_estimate: 60
- shell_to_rust_ratio_delta_estimate: +0.0003
- shell_surface_mitigation_issue: None

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | checker + valid dry-run report | checker returns pass/GO with deterministic markers |
| C-02 | AC-1 | Regression | checker + malformed/threshold-violating report | checker fails closed with deterministic reason codes |
| C-03 | AC-2 | Unit | threshold fixture parsing | required keys parse and invalid/missing keys fail closed |
| C-04 | AC-3 | Integration | checker + `scripts/ci/test_ci_tools.sh` fast-mode command surface | required overload test entry exists and direct heavy-run entry is blocked |
| C-05 | AC-4 | Conformance | targeted shell tests + docs-contract tests | all checker/selector/docs tests pass |

## Test Mapping
- C-01 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- C-02 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- C-03 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- C-04 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh` + `bash scripts/ci/test_ci_tools.sh`
- C-05 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh` + `cargo test -p kamn-core --test ci_strategy_docs`

## Success Metrics
- Checker and thresholds detect drift deterministically.
- Fast-gate command surface keeps local-heavy overload run path excluded.
- CI docs markers and tests stay synchronized with checker behavior.
