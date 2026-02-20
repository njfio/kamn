# Issue #5408 Plan

- Issue: #5408
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Implementation Approach

1. RED first:
- add a new runtime shell test that asserts dispatcher-symlink + exec-registry mapping for the crash-restart runner wrapper.
- run the new test and capture failing output against current inline-shell implementation.

2. GREEN implementation:
- add `scripts/runtime/sqlite_crash_restart_local_heavy_lane_contract.py` with the existing lane projection logic.
- convert `scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh` to an exec-dispatch symlink.
- add registry entry in `scripts/lib/exec_registry.json` for the runner wrapper.
- wire the new shell regression test into `scripts/ci/test_ci_tools.sh` fast/full paths.

3. VERIFY:
- rerun new shell regression test and existing sqlite crash-restart contract tests.
- run formatting/lint gates touched by the change set.

## Affected Modules

- `specs/5408/spec.md`
- `specs/5408/plan.md`
- `specs/5408/tasks.md`
- `scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh`
- `scripts/runtime/sqlite_crash_restart_local_heavy_lane_contract.py` (new)
- `scripts/runtime/test_run_sqlite_crash_restart_local_heavy_lane.sh` (new)
- `scripts/lib/exec_registry.json`
- `scripts/ci/test_ci_tools.sh`

## Risks and Mitigations

- Risk: marker drift during migration from shell to Python.
  - Mitigation: preserve canonical constants and assert marker parity in the new shell regression test + existing Rust tests.
- Risk: wrapper dispatch wiring drift.
  - Mitigation: explicit registry/symlink assertions in the new test.
- Risk: CI command-surface regression.
  - Mitigation: wire the new test into both fast/full `test_ci_tools.sh` execution paths.

## Interface / Contract Markers

- lane report schema: `kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1`
- artifact schema: `kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1`
- reason taxonomy: `kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1`
- reason codes CSV: `crash_restart_profile_restart_status_mismatch,crash_restart_profile_corruption_status_mismatch,crash_restart_profile_combined_status_mismatch`

## ADR

- Not required (implementation-surface migration only; no protocol/dependency changes).
