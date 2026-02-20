# Issue #5408 Spec

- Title: Task: reduce shell surface for crash-restart local-heavy lane runner
- Status: Implemented
- Type: task
- Priority: P2
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Problem Statement

`scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh` currently embeds runner + projection logic in a large shell script with inline Python, which unnecessarily increases shell surface for a deterministic contract lane already governed by policy tests.

## Scope

In scope:
- migrate crash-restart local-heavy lane runner implementation out of the shell wrapper into a dedicated Python contract module.
- convert the shell entrypoint into a dispatcher-backed wrapper.
- add explicit regression coverage for wrapper dispatch/registry wiring and marker stability.

Out of scope:
- reason-taxonomy or policy-checker behavior changes.
- docs/runbook marker contract changes.

## Shell-Surface Estimates

- shell_loc_delta_estimate: -190
- rust_loc_delta_estimate: 0
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria

- AC-1: `scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh` becomes a dispatcher-backed wrapper with a registry entry targeting a Python implementation module.
- AC-2: Existing deterministic lane markers and JSON report schema values remain unchanged for dry-run profiles.
- AC-3: A dedicated regression test fails closed on wrapper/registry drift and passes on the migrated surface.
- AC-4: Shell LOC contribution from the crash-restart lane runner is measurably reduced while preserving existing contract test pass paths.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-3 | Conformance | wrapper + exec registry | wrapper resolves through dispatcher, registry target/args contract holds |
| C-02 | AC-2 | Functional | `--profile combined --mode dry-run` | deterministic markers and schema values preserved |
| C-03 | AC-2 | Integration | existing rust lane/policy contract tests | unchanged `main_tests::daemon_tests`/sqlite crash-restart contract behavior |
| C-04 | AC-4 | Conformance | line-count measurement | shell LOC for runner wrapper reduced from baseline |

## Test Mapping

- `bash scripts/runtime/test_run_sqlite_crash_restart_local_heavy_lane.sh`
- `bash scripts/runtime/test_check_sqlite_crash_restart_local_heavy_policy.sh`
- `cargo test -p kamn-core --test sqlite_crash_restart_local_heavy_lane_contract -- --nocapture`
- `cargo test -p kamn-core --test sqlite_crash_restart_local_heavy_policy_contract -- --nocapture`

## Success Metrics / Observable Signals

- crash-restart lane runner behavior remains deterministic and policy-compatible.
- wrapper implementation shifts from high-shell surface to registry-backed Python implementation.
- CI tooling retains explicit drift detection for wrapper/registry contracts.
