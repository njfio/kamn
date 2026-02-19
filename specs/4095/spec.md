# Issue #4095 Spec

- Title: Subtask: add degradation marker policy checker and taxonomy drift checks for overload governance
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Problem Statement
The daemon OS-signal stress matrix report currently emits decision markers but does not project explicit taxonomy metadata into the report payload itself. This leaves a drift gap where report reason taxonomy can diverge from docs/checker thresholds without a deterministic fail-closed signal.

## Acceptance Criteria
- AC-1: `run_daemon_os_signal_stress_matrix.sh` emits deterministic overload taxonomy markers (`reason_taxonomy_version`, `reason_codes_csv`) in report JSON and stdout.
- AC-2: `check_daemon_os_signal_stress_policy.py` validates stress report taxonomy fields against threshold fixture expectations and fails closed on mismatch.
- AC-3: Overload governance tests include explicit pass/fail scenarios for taxonomy-version and reason-csv drift.
- AC-4: CI/ops docs remain aligned with overload taxonomy markers and checker expectations.

## Scope
In scope:
- Extend overload stress matrix report schema with taxonomy metadata markers.
- Extend existing dry-run overload checker threshold contract with taxonomy drift keys.
- Add/extend tests for taxonomy drift fail-closed behavior.
- Update strategy docs markers for threshold/checker taxonomy keys.
- Add `specs/4095/{spec.md,plan.md,tasks.md}`.

Out of scope:
- New local-heavy runtime lanes.
- Runtime daemon behavior changes unrelated to overload-marker policy projection.
- CI workflow topology changes.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 40
- rust_loc_delta_estimate: 20
- shell_to_rust_ratio_delta_estimate: +0.0001
- shell_surface_mitigation_issue: None

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | stable stress matrix run | report and stdout include overload taxonomy version and reason csv markers |
| C-02 | AC-2 | Functional | checker + valid report/threshold | checker passes with deterministic `status=pass` and `reason_codes=none` |
| C-03 | AC-2 | Regression | checker + taxonomy mismatch report | checker fails closed with deterministic taxonomy mismatch reason code |
| C-04 | AC-2 | Regression | checker + reason-csv mismatch report | checker fails closed with deterministic reason-csv mismatch reason code |
| C-05 | AC-3 | Integration | stress runner + checker + fixture composition | composed policy path remains pass for valid baseline and fail for drift |
| C-06 | AC-4 | Conformance | docs-contract overload marker tests | docs remain synchronized with checker/runner markers |

## Test Mapping
- C-01 -> `bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh`
- C-02 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- C-03 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- C-04 -> `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- C-05 -> `bash scripts/ci/test_run_daemon_os_signal_stress_matrix.sh` + `bash scripts/ci/test_check_daemon_os_signal_stress_policy.sh`
- C-06 -> `cargo test -p kamn-core --test ci_strategy_docs doc_contains_overload_ci_dry_run_policy_checker_markers -- --exact`

## Success Metrics
- Stress report, checker threshold fixture, and docs project one deterministic overload taxonomy contract.
- Taxonomy drift is detected in a single checker invocation with fail-closed reason markers.
- Fast-gate checker cost remains low and local-heavy execution remains opt-in only.
