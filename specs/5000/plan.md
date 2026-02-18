# Issue #5000 Plan

- Issue: #5000
- Status: Implemented

## Approach (Implemented)
1. Verify and finalize docs-contract remediation by replacing deleted wrapper references with manifest-runner command strings in both docs and assertion tests.
2. Run scoped docs-contract suite to confirm deterministic green state.
3. Execute archive wave 2 with `archive_completed_specs.py` and capture moved-id count.
4. Run archive-policy checks and fix any parity/index drift detected by fail-closed scripts.
5. Synchronize issue/spec lifecycle artifacts to implemented state with measured shell-surface deltas.

## Affected Modules
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/archive/**`
- `specs/*/ARCHIVED.md` (wave-2 candidates)
- `specs/archive/index.md` (if wave metadata updates are generated)
- `specs/5000/spec.md`
- `specs/5000/tasks.md`

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Large archive move may leave stale pointers or index mismatch.
  - Docs/test command strings can drift if manifest names are mistyped.
- Mitigations:
  - Run fail-closed archive checker immediately after wave execution.
  - Keep command strings copied from actual manifest-runner surfaces and verify via targeted tests.

## Interface Contract
- No protocol/wire-format/API behavior change in this task.
- Archive tooling contract remains:
  - active `specs/<id>/` for archived entries contains pointer file `ARCHIVED.md`,
  - canonical content resides in `specs/archive/<id>/`,
  - checker emits deterministic reason taxonomy when parity fails.

## ADR
- No ADR required (process/docs/archive operations only).
