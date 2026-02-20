# Issue #4017 Plan

- Issue: #4017
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Implementation Approach

1. RED first:
- add `crates/kamn-core/tests/sqlite_crash_restart_local_heavy_lane_contract.rs` with failing runner contract assertions.
- add ops-doc marker assertion in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`.
- run targeted RED test command.

2. GREEN implementation:
- add runner wrapper `scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh` that:
  - enforces profile contract (`restart|corruption|combined`),
  - executes existing sqlite crash-recovery contract lane,
  - emits deterministic artifact schema/report markers.
- update `docs/ops/configuration.md` with marker table and command mapping.

3. VERIFY:
- rerun targeted tests.
- run `cargo fmt --check` and scoped `cargo clippy`.

## Affected Modules

- `specs/4017/spec.md`
- `specs/4017/plan.md`
- `specs/4017/tasks.md`
- `scripts/runtime/run_sqlite_crash_restart_local_heavy_lane.sh` (new)
- `crates/kamn-core/tests/sqlite_crash_restart_local_heavy_lane_contract.rs` (new)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ops/configuration.md`

## Risks and Mitigations

- Risk: wrapper marker drift from source lane output.
  - Mitigation: integration contract asserts schema and source-marker mapping.
- Risk: local-heavy command runtime variance.
  - Mitigation: dry-run budget assertion and bounded runtime marker.
- Risk: docs drift.
  - Mitigation: exact docs marker assertions in ops docs tests.

## Interface / Contract Markers

- runner report schema:
  - `kamn.runtime.sqlite-crash-restart-local-heavy-lane-report.v1`
- runner artifact schema:
  - `kamn.runtime.sqlite-crash-restart-local-heavy-artifact-schema.v1`
- reason taxonomy:
  - `kamn.runtime.sqlite-crash-restart-local-heavy-reason-taxonomy.v1`

## ADR

- Not required (wrapper contract + tests/docs only; no dependency/protocol change).
