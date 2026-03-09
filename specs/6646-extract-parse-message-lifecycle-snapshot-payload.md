# 6646 - Extract parse_message_lifecycle_snapshot_payload into bounded helpers

## Objective

Reduce `parse_message_lifecycle_snapshot_payload()` in `crates/kamn-core/src/message_lifecycle.rs` to a small coordinator that delegates schema parsing, record-field parsing, list decoding, and snapshot assembly to clearly named helpers without changing the payload format or external API behavior.

## Inputs/Outputs

### Inputs
- `crates/kamn-core/src/message_lifecycle.rs`
- Existing snapshot serialization/parsing helpers in the same module
- Existing snapshot store regression coverage in `crates/kamn-core/src/message_lifecycle.rs` tests

### Outputs
- A small `parse_message_lifecycle_snapshot_payload()` coordinator function
- Extracted helper functions for schema parsing, record line parsing, status/history decoding, and record construction
- Direct regression tests for happy-path parsing and malformed payload failure cases
- Preserved `MessageLifecycleSnapshotStoreError::InvalidPayload` error semantics for corrupt payloads

## Boundaries/Non-goals

- Do not change the on-disk snapshot payload format
- Do not change public APIs or external error types
- Do not redesign the broader message lifecycle store or journal flow
- Do not split `message_lifecycle.rs` into new files in this issue

## Failure Modes

- Missing or malformed schema line stops parsing with `InvalidPayload`
- Non-`record` line prefixes stop parsing with `InvalidPayload`
- Record lines with missing or extra fields stop parsing with `InvalidPayload`
- Invalid status codes in `status` or `history` stop parsing with `InvalidPayload`
- Refactor drifts store/journal restore behavior by changing accepted payload shapes

## Acceptance Criteria

- [ ] `parse_message_lifecycle_snapshot_payload()` is reduced to a small coordinator function
- [ ] Schema parsing, record parsing, and status/history decoding are extracted into clearly named helper functions
- [ ] Helper functions preserve current `InvalidPayload` failure semantics for malformed lines
- [ ] Direct regression tests cover a valid payload plus malformed schema, malformed record field count, and invalid status/history payloads
- [ ] Existing snapshot store round-trip and malformed-payload tests remain green
- [ ] No new extracted helper exceeds the repo 25 LOC function limit unless explicitly staged and justified in this spec

## Files To Touch

- `specs/6646-extract-parse-message-lifecycle-snapshot-payload.md`
- `crates/kamn-core/src/message_lifecycle.rs`

## Error Semantics

- Malformed payloads continue to return `MessageLifecycleSnapshotStoreError::InvalidPayload`
- Helpers fail closed and bubble invalid line contents back through the existing error type
- No silent fallback or payload normalization is introduced

## Test Plan

1. Red: add direct parser tests in `message_lifecycle.rs` for valid payload round-trip, malformed schema, malformed record field count, and invalid status/history lines
2. Green: extract helpers for schema parsing, record parsing, list decoding, and snapshot assembly until the new tests and existing malformed-payload/store tests pass
3. Refactor: trim duplication in invalid-line error construction and keep each helper within the function budget
4. Integration: rerun the parser-focused tests plus existing snapshot store round-trip/malformed recovery tests that exercise the real file/journal restore paths

## Notes / Deviations

- The current function is 122 LOC on `origin/main`, not ~702 LOC as stated in the issue body, but it still violates the 25 LOC function policy and has obvious extraction seams.
