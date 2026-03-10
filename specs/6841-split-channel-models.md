# 6841-split-channel-models

## Objective
Reduce `crates/kamn-core/src/channel_models.rs` from a 1780 LOC production monolith to a thin root shell plus bounded concern modules without changing channel model behavior, snapshot semantics, snapshot-store behavior, or exported API surface.

## Inputs/Outputs
- Input: existing channel types, membership/admin mutation logic, snapshot codec logic, file/sqlite snapshot stores, journal replay helpers, and inline tests in `crates/kamn-core/src/channel_models.rs`
- Output: bounded sibling modules and a root shell that preserves the public `kamn_core::channel_models` surface
- Output: structural ratchet coverage that enforces the staged extraction layout on touched code

## Boundaries/Non-goals
- No new dependencies
- No public API behavior changes beyond module movement required for extraction
- No changes to channel membership/admin policy semantics
- No changes to snapshot schema, journal encoding, or sqlite persistence behavior
- No weakening of existing tests or invariants

## Failure Modes
- Root shell remains above the staged extraction cap
- Any touched extracted file exceeds 200 LOC
- Public exports drift and downstream crates fail to compile
- Snapshot validation, journal replay, or sqlite persistence semantics change during extraction
- Touched-Rust size policy remains `NO-GO`

## Acceptance Criteria
- [x] `crates/kamn-core/src/channel_models.rs` is reduced to a thin root shell at or below a staged extraction cap defined by the red contract
- [x] Extracted sibling modules are organized by concern rather than arbitrary line slicing
- [x] All touched extracted files remain at or below 200 LOC
- [x] Existing channel model behavior, snapshot semantics, and snapshot-store behavior remain unchanged
- [x] Existing tests that exercise the channel model surface still pass
- [x] At least one new extraction contract enforces the staged root shell and module layout
- [x] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root <repo-root> --base-ref origin/main --output-json <path>` returns `policy_decision=GO`

## Files To Touch
- `specs/6841-split-channel-models.md`
- `crates/kamn-core/src/channel_models.rs`
- `crates/kamn-core/src/channel_models/`
- `crates/kamn-core/tests/channel_models_module_extraction_contract.rs`

## Error Semantics
- Preserve all existing `ChannelModelError`, `ChannelSnapshotError`, and `ChannelSnapshotStoreError` variants and displayed messages unless a failing test proves the old behavior was already wrong
- Preserve current fail-closed behavior for invalid channel metadata, membership/admin state, malformed snapshots, corrupt journal tails, and sqlite payload errors
- Structural contract failures must fail with explicit missing-file, missing-marker, or root-budget assertions

## Test Plan
1. Add a red extraction contract for `crates/kamn-core/src/channel_models.rs`.
2. Split the file into bounded modules by concern:
   - channel types/records/errors
   - channel store creation/membership/admin mutation logic
   - snapshot validation and codec helpers
   - file/sqlite snapshot-store adapters and journal replay helpers
   - internal tests moved under `src/channel_models/tests/` while preserving stable paths if needed
3. Run the extraction contract and the existing `channel_models` test lane.
4. Run the touched-Rust size ratchet and require `policy_decision=GO`.

## Phase 6 Evidence
- Root shell and real entrypoint preserved through `crates/kamn-core/src/channel_models.rs`, which still owns the exported `kamn_core::channel_models` surface while delegating to bounded child modules.
- Verified real path:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core channel_models::tests:: -- --nocapture`
- Verified structural contract:
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test channel_models_module_extraction_contract -- --nocapture`
- Verified touched-Rust gate:
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6840-remote --base-ref origin/main --output-json /tmp/6841-touched-size-final.json`
  - result: `policy_decision=GO`

## Deviations
- None.
