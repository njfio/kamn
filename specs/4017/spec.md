# Issue #4017 Spec

- Title: Subtask: implement crash-restart local-heavy lane runner with deterministic recovery artifacts
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Problem Statement

Sqlite crash-recovery durability infrastructure exists, but operators still need a dedicated local-heavy crash-restart lane runner that exposes explicit restart/corruption drill profiles through a deterministic artifact schema. Without this profile-oriented wrapper contract, local drill execution and evidence consumption remain less reproducible than required.

## Scope

In scope:
- Add local-heavy crash-restart lane runner wrapper with deterministic profile surface (`restart`, `corruption`, `combined`).
- Emit deterministic recovery artifact schema and reason taxonomy markers.
- Add contract tests for runner profile behavior, failure paths, and runtime budget.
- Update `docs/ops/configuration.md` with crash-restart artifact marker table and validation command.
- Add docs parity assertions in `service_api_ops_configuration_docs`.

Out of scope:
- Running local-heavy lane in CI fast-gate defaults.
- External runbook platform migration.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 120
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0004
- shell_surface_mitigation_issue: None

## Acceptance Criteria

- AC-1: Runner emits deterministic recovery artifact schema for `restart`, `corruption`, and `combined` profiles.
- AC-2: Restart/corruption drill profiles are bounded and reproducible in dry-run mode.
- AC-3: Runner fails closed on invalid profile/mode inputs and projects deterministic reason markers.
- AC-4: `docs/ops/configuration.md` contains crash-restart artifact marker table and validation commands; docs contract fails closed on drift.
- AC-5: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | runner `--profile combined --mode dry-run` | deterministic schema/taxonomy markers and `status=pass` |
| C-02 | AC-1/AC-2 | Functional | runner `--profile restart --mode dry-run` | restart profile view is deterministic and bounded |
| C-03 | AC-1/AC-2 | Functional | runner `--profile corruption --mode dry-run` | corruption profile view is deterministic and bounded |
| C-04 | AC-1 | Integration | runner `--profile combined` + artifact JSON | recovery artifact fields/projected source markers are deterministic |
| C-05 | AC-3 | Regression | runner invalid profile input | fail closed with deterministic validation error |
| C-06 | AC-5 | Performance | runner dry-run execution | bounded local runtime under CI budget |
| C-07 | AC-4 | Regression | ops docs marker assertions | docs parity drift fails closed (`Regression: #4017`) |

## Test Mapping

- `cargo test -p kamn-core --test sqlite_crash_restart_local_heavy_lane_contract -- --nocapture`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_sqlite_crash_restart_local_heavy_lane_markers -- --exact`
- `cargo fmt --check`
- `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics / Observable Signals

- Local-heavy crash-restart runner is profile-driven and deterministic.
- Artifact schema markers are stable and consumption-ready for policy checker composition.
- Ops docs and docs-contract tests stay synchronized with runner markers.
