## Objective

Add explicit integration coverage for `kamn-snapshot-journal` append semantics and corrupted
record handling so newline-delimited persistence and fail-closed decode behavior are specified by
tests.

## Inputs/Outputs

- Inputs:
  - `append_snapshot_journal_record(journal_path, payload)`
  - `parse_snapshot_journal_record(line)`
  - `parse_snapshot_journal_record_checked(line)`
  - `decode_snapshot_journal_hex(value)`
  - `default_snapshot_journal_path(path)`
- Outputs:
  - deterministic newline-delimited append behavior across multiple records
  - deterministic parse/decode behavior for uppercase hex and corrupted JSON record payloads

## Boundaries/Non-goals

- No production API changes
- No workflow, CI, or dependency changes
- No changes to schema versioning or file naming
- No stress/performance benchmark additions in this slice

## Failure modes

- Appending multiple records overwrites prior journal entries or omits newline delimiters
- Uppercase hex payloads stop decoding successfully
- A valid JSON record with corrupted `payload_hex` stops failing closed during decode
- Records missing required JSON fields stop failing checked parse as invalid JSON

## Acceptance criteria

- [ ] A test proves appending multiple snapshot records preserves record order
- [ ] A test proves appending multiple snapshot records produces newline-delimited journal lines
- [ ] A test proves uppercase hex payloads decode successfully
- [ ] A test proves a valid JSON record with corrupted `payload_hex` parses but decode fails with `None`
- [ ] A test proves records missing `schema_version` fail checked parse with `InvalidJson`
- [ ] A test proves records missing `payload_hex` fail checked parse with `InvalidJson`
- [ ] `cargo test -p kamn-snapshot-journal -- --nocapture` passes

## Files to touch

- `specs/6483-add-snapshot-journal-corruption-and-append-coverage.md`
- `crates/kamn-snapshot-journal/tests/snapshot_journal_edge_cases_contract.rs`
- `crates/kamn-snapshot-journal/tests/snapshot_journal_edge_cases.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` (only if new test targets change inventory)

## Error semantics

- `parse_snapshot_journal_record_checked(...)` continues to return
  `SnapshotJournalParseError::InvalidJson` for structurally invalid or field-missing JSON records
- `decode_snapshot_journal_hex(...)` continues to fail closed with `None` for corrupted hex input
- No new error variants are introduced

## Test plan

- Add a contract test that requires a dedicated edge-case integration target
- Add integration tests for multi-record append ordering, newline delimitation, uppercase hex
  decode, corrupted `payload_hex` decode failure, and missing-field checked parse failures
- Run:
  - `cargo test -p kamn-snapshot-journal --test snapshot_journal_edge_cases_contract -- --nocapture`
  - `cargo test -p kamn-snapshot-journal --test snapshot_journal_edge_cases -- --nocapture`
  - `cargo test -p kamn-snapshot-journal -- --nocapture`
  - `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` if test inventory changes

## Deviations

- None
