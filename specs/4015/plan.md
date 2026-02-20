# Issue #4015 Plan

- Issue: #4015
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Implementation Approach

1. RED first:
- add `crates/kamn-core/tests/journal_wal_commit_schema_contract.rs` with fixture parser + store contract assertions.
- add docs assertion in `crates/kamn-core/tests/runtime_network_docs.rs` for a new journal/WAL marker rules section.
- run targeted tests to capture failing RED evidence before fixture/docs updates.

2. GREEN fixture/docs:
- add `fixtures/runtime/journal_wal_commit_boundary_fixture_matrix.txt` containing deterministic schema/marker metadata and per-store rows.
- update `docs/foundation/runtime-network.md` with journal/WAL commit-boundary marker map and regression marker.

3. VERIFY:
- rerun targeted contract tests and docs exact test.
- run `cargo fmt --check` and `cargo clippy -p kamn-core --tests -- -D warnings`.

## Affected Modules

- `specs/4015/spec.md`
- `specs/4015/plan.md`
- `specs/4015/tasks.md`
- `crates/kamn-core/tests/journal_wal_commit_schema_contract.rs` (new)
- `fixtures/runtime/journal_wal_commit_boundary_fixture_matrix.txt` (new)
- `docs/foundation/runtime-network.md`
- `crates/kamn-core/tests/runtime_network_docs.rs`

## Risks and Mitigations

- Risk: fixture markers diverge from store behavior.
  - Mitigation: integration assertions invoke real file-backed stores and validate recovery reason codes directly.
- Risk: docs drift from fixture contracts.
  - Mitigation: exact docs marker assertions in `runtime_network_docs` fail closed.
- Risk: overfitting to one payload sample.
  - Mitigation: per-store integration checks validate structure and deterministic marker map rather than brittle payload string equality.

## Interface / Contract Markers

- Fixture schema marker:
  - `journal_wal_fixture_matrix_schema_version=kamn.runtime.journal-wal-fixture-matrix.v1`
- Journal entry schema marker:
  - `journal_entry_shape=entry|1|<payload_hex>`
- Commit-boundary taxonomy marker:
  - `commit_boundary_marker_taxonomy_version=kamn.runtime.snapshot-journal-commit-boundary-markers.v1`

## ADR

- Not required (no dependency, protocol, or wire-format change).
