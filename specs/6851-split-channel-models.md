# Objective
Split `crates/kamn-core/src/channel_models.rs` into bounded, concern-based modules while preserving channel membership/admin policy, snapshot export and restore semantics, snapshot-store behavior, and the existing channel-model test surface.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/channel_models.rs`
  - existing channel snapshot journal helpers and sqlite/file store integrations
  - current `channel_models` and snapshot-store tests
- Outputs:
  - a thin `channel_models.rs` root shell under the active file-size budget
  - bounded sibling modules for store operations, snapshot-store backends, snapshot codec/journal helpers, errors, and tests
  - extraction contract coverage enforcing the new module layout and active size limits

## Boundaries/Non-goals
- Do not change channel membership/admin semantics.
- Do not change channel snapshot schema or journal format.
- Do not rework unrelated message or task-operation logic.
- Do not introduce new dependencies.

## Failure modes
- Root file remains oversized after the split.
- Extracted modules drift direct/group/broadcast/task/marketplace/governance channel semantics.
- Snapshot export/restore or journal replay behavior changes.
- Extracted files or functions still exceed active size limits.

## Acceptance criteria
- [ ] The root file is reduced to a thin shell under the active file-size budget.
- [ ] Channel store, snapshot store, snapshot codec, error, and test seams are extracted into bounded modules.
- [ ] Existing `channel_models` tests remain green.
- [ ] No extracted file exceeds the active file-size limit.
- [ ] No extracted function exceeds the active function-size limit.

## Files to touch
- `specs/6851-split-channel-models.md`
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/tests/channel_models_module_extraction_contract.rs`
- optional sibling modules under `crates/kamn-core/src/channel_models/`

## Error semantics
- Existing `ChannelModelError`, `ChannelSnapshotError`, and `ChannelSnapshotStoreError` behavior remains fail-closed.
- Snapshot schema validation, malformed payload detection, and journal-corrupt-tail handling remain deterministic.
- No silent fallback behavior may be introduced during the split.

## Test plan
1. Add a red extraction contract that fails while the root file remains oversized and the planned module layout is absent.
2. Re-run the existing `channel_models` tests that cover membership/admin policy, snapshot export/restore, and store backends.
3. Extract the file into bounded concern-based modules.
4. Re-run the extraction contract and real `channel_models` targets until green.
5. Run the clean-clone touched-Rust size ratchet on the final write set.
