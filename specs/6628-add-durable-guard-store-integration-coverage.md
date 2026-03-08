# 6628-add-durable-guard-store-integration-coverage

## Objective
Add dedicated crate-level integration coverage for the public `durable_guard_store` boundary so bundle capture/restore and the in-memory/file-backed persistence lanes remain pinned outside inline module tests.

## Inputs/Outputs
- Inputs:
  - public `DurableGuardSnapshotBundle`, `InMemoryDurableGuardSnapshotStore`, `FileDurableGuardSnapshotStore`, `MessageDeliveryGuards`, and `ChannelPermissionEngine` values
  - invalid bundle schema and invalid file payload values through the public API
- Outputs:
  - dedicated integration test surface at `crates/kamn-core/tests/durable_guard_store_integration.rs`
  - dedicated contract test at `crates/kamn-core/tests/durable_guard_store_contract.rs`
  - refreshed `test_file_size_policy` baseline if the new test targets change workspace inventory counts

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-core/src/durable_guard_store.rs`
- No sqlite lane coverage in this issue
- No CI or workflow changes
- No visibility changes for internal helpers solely to support tests

## Failure modes
- Dedicated durable-guard integration surface missing entirely
- bundle capture/restore stops reproducing delivery and channel guard state into live engines
- in-memory save/load stops round-tripping validated bundles
- file-backed save/load stops round-tripping validated bundles
- invalid bundle schema stops failing closed
- invalid on-disk payload stops failing closed
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `durable_guard_store_contract` fails when the dedicated integration surface or its marker cases disappear
- [ ] integration coverage asserts bundle capture and restore reproduce delivery nonce/replay state and channel snapshot state through the public API
- [ ] integration coverage asserts the in-memory store save/load lane round-trips validated bundles
- [ ] integration coverage asserts the file-backed store save/load lane round-trips validated bundles from disk
- [ ] integration coverage asserts invalid bundle schema and invalid file payloads fail closed through the public API
- [ ] `cargo test -p kamn-core --test durable_guard_store_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test durable_guard_store_integration -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6628-add-durable-guard-store-integration-coverage.md`
- `crates/kamn-core/tests/durable_guard_store_integration.rs`
- `crates/kamn-core/tests/durable_guard_store_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing durable-guard store fail-closed behavior only
- Invalid bundle schema and invalid payload paths must preserve current `DurableGuardSnapshotStoreError` values
- No new production error types or translation layers are introduced

## Test plan
1. Add a contract test referencing `durable_guard_store_integration.rs` before that file exists so the red phase is a real missing-surface failure.
2. Add a dedicated integration surface covering bundle capture/restore, in-memory persistence, file persistence, and fail-closed invalid schema/payload cases through the public API.
3. Run targeted durable-guard contract and integration tests.
4. Run `test_file_size_policy` and refresh its baseline only if the new test targets change inventory counts.

## Phase 6 notes
- No production durable-guard-store behavior changes were required.
- Integration for this issue is the dedicated crate-level durable-store surface plus the contract pin that prevents silent removal of that surface.
- Adding the two new test targets increased the workspace `test_file_total` baseline from `479` to `481`.
