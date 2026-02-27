# Plan: Issue #6112

## Approach
1. Add a small shared module under `kamn-core` for journal helper primitives.
2. Move duplicated `encode_journal_hex` / `decode_journal_hex` / nibble decode logic to shared helpers.
3. Update message/channel/task snapshot-store internals to call shared helpers.
4. Keep payload format unchanged and preserve current error semantics.
5. Run focused tests for the three refactored modules.

## Affected Modules
- `crates/kamn-core/src/message_lifecycle.rs`
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/src/task_operations.rs`
- `crates/kamn-core/src/lib.rs` (module export)
- `crates/kamn-core/src/<new-shared-module>.rs`

## Risks
- Risk: subtle behavior drift in invalid-payload handling.
  - Mitigation: preserve Option-based decode contract and add direct helper tests.
- Risk: format drift in persisted data.
  - Mitigation: no format changes; only helper indirection.

## Interfaces/Contracts
- Internal-only refactor; no public API changes expected.
