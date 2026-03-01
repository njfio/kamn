# Spec: Issue #6289 - Snapshot-journal typed parse errors + integration lane

## Objective

Add a typed parse API to `kamn-snapshot-journal` so invalid records fail with explicit reasons and
establish first crate-level integration coverage.

## Inputs/Outputs

- Inputs:
  - Journal JSON line string.
- Outputs:
  - Success: `Ok(payload_hex)`
  - Failure: `SnapshotJournalParseError` with one of:
    - `InvalidJson`
    - `SchemaVersionMismatch`
    - `MissingPayloadHex`

## Boundaries/Non-goals

- In scope:
  - New typed parse function + error enum in `src/lib.rs`.
  - Keep compatibility `parse_snapshot_journal_record` API.
  - Add integration tests in `tests/`.
- Out of scope:
  - Schema version changes.
  - Write-path behavior changes.
  - Cross-crate wiring.

## Failure Modes

- FM-1: invalid JSON line fails without reason specificity.
- FM-2: schema mismatch or empty payload is not distinguishable.
- FM-3: compatibility API regression.

## Acceptance Criteria

- AC-1: `parse_snapshot_journal_record_checked(&str) -> Result<String, SnapshotJournalParseError>`
  exists.
- AC-2: invalid JSON returns `InvalidJson`.
- AC-3: schema mismatch returns `SchemaVersionMismatch`.
- AC-4: empty payload hex returns `MissingPayloadHex`.
- AC-5: `parse_snapshot_journal_record` remains and delegates to checked API.
- AC-6: integration tests under `crates/kamn-snapshot-journal/tests/` cover success + all failure
  modes.

## Files To Touch

- `crates/kamn-snapshot-journal/src/lib.rs`
- `crates/kamn-snapshot-journal/tests/snapshot_journal_integration.rs`
- `specs/6289-snapshot-journal-typed-parse-errors.md`

## Error Semantics

- Parse boundary returns typed errors only (no silent fallback).
- Compatibility API intentionally preserves legacy `Option` behavior by dropping typed error detail.

## Test Plan

- RED:
  - Add integration tests for checked parse API (success + each error variant).
  - Confirm tests fail before implementation.
- GREEN:
  - Implement checked parse API and compatibility delegation.
- REFACTOR:
  - Keep parse helper small and deterministic.
- Verification:
  - `cargo fmt --all --check`
  - `cargo clippy -p kamn-snapshot-journal --tests -- -D warnings`
  - `cargo test -p kamn-snapshot-journal`
