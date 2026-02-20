# Issue #4016 Plan

- Issue: #4016
- Milestone: specs/milestones/r27-10-durability-crash-recovery-and-state-consistency-hardening/index.md

## Implementation Approach

1. RED first:
- add `crates/kamn-core/tests/journal_wal_partial_write_fault_contract.rs` with fixture parser + partial-write fault matrix assertions.
- add docs-contract test assertion in `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` for new ops markers.
- run targeted tests and capture RED failure before fixture/docs/logic changes.

2. GREEN implementation:
- update file-backed snapshot stores (`channel_models`, `message_lifecycle`, `task_operations`) so valid journal replay is authoritative before snapshot-file parse fallback.
- add fixture `fixtures/runtime/journal_wal_partial_write_fault_matrix.txt`.
- update `docs/ops/configuration.md` with partial-write drill markers, inputs, outcomes, and validation commands.

3. VERIFY:
- rerun targeted test suites.
- run `cargo fmt --check` and scoped `clippy`.

## Affected Modules

- `specs/4016/spec.md`
- `specs/4016/plan.md`
- `specs/4016/tasks.md`
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/src/task_operations.rs`
- `crates/kamn-core/tests/journal_wal_partial_write_fault_contract.rs` (new)
- `fixtures/runtime/journal_wal_partial_write_fault_matrix.txt` (new)
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations

- Risk: changing read precedence could mask malformed journal behavior.
  - Mitigation: journal replay remains fail-closed; only snapshot-file fallback is deferred until journal absence.
- Risk: cross-store behavior drift.
  - Mitigation: fixture-driven matrix asserts identical fault-mode semantics across all three stores.
- Risk: ops docs drift from contracts.
  - Mitigation: add exact docs marker assertions in `service_api_ops_configuration_docs`.

## Interface / Contract Markers

- Fixture schema marker:
  - `journal_wal_partial_write_fixture_schema_version=kamn.runtime.journal-wal-partial-write-fault-matrix.v1`
- Fault taxonomy marker:
  - `journal_wal_partial_write_reason_taxonomy_version=kamn.runtime.journal-wal-partial-write-reason-taxonomy.v1`

## ADR

- Not required (no dependency/protocol/wire-format change).
