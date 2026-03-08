# 6624-add-runtime-guard-message-delivery-coverage

## Objective
Add dedicated crate-level integration coverage for the public `kamn_runtime_guards::message_delivery_guards` boundary so replay rejection, nonce-state progression, and snapshot restoration remain pinned outside inline module tests.

## Inputs/Outputs
- Inputs:
  - public `DeliveryGuardInput`, `MessageDeliveryGuards`, and `DeliveryGuardSnapshot` values covering valid and fail-closed cases
  - sequential deliveries and restored snapshot state through the public API
- Outputs:
  - dedicated integration test surface at `crates/kamn-runtime-guards/tests/runtime_guard_message_delivery.rs`
  - dedicated contract test at `crates/kamn-runtime-guards/tests/runtime_guard_message_delivery_contract.rs`
  - refreshed `test_file_size_policy` baseline if the new test targets change workspace inventory counts

## Boundaries/Non-goals
- No production behavior changes in `crates/kamn-runtime-guards/src/message_delivery_guards.rs`
- No changes to anti-spam, quota, fairness, watchdog, retention, or policy-stack modules
- No CI or workflow changes
- No visibility changes for internal helpers solely to support tests

## Failure modes
- Dedicated message-delivery integration surface missing entirely
- first valid delivery stops being accepted and incrementing sender nonce state
- replayed message ids stop failing closed with deterministic rejection notices
- nonce floor/sequence enforcement stops failing closed
- restored snapshots stop reproducing expected nonce/replay state
- invalid snapshot payloads stop failing closed
- workspace `test_file_size_policy` inventory drifts after adding new test targets

## Acceptance criteria (testable booleans)
- [ ] `runtime_guard_message_delivery_contract` fails when the dedicated integration surface or its marker cases disappear
- [ ] integration coverage asserts a first valid delivery is accepted and increments expected nonce
- [ ] integration coverage asserts replay rejection and nonce-out-of-sequence rejection preserve deterministic failure codes/signatures
- [ ] integration coverage asserts snapshot export/restore reproduces nonce and replay state across a new guard instance
- [ ] integration coverage asserts invalid snapshot payloads fail closed through the public API
- [ ] `cargo test -p kamn-runtime-guards --test runtime_guard_message_delivery_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-runtime-guards --test runtime_guard_message_delivery -- --nocapture` passes
- [ ] `cargo test -p kamn-runtime-guards -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` passes

## Files to touch
- `specs/6624-add-runtime-guard-message-delivery-coverage.md`
- `crates/kamn-runtime-guards/tests/runtime_guard_message_delivery.rs`
- `crates/kamn-runtime-guards/tests/runtime_guard_message_delivery_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` if needed

## Error semantics
- Tests assert the existing public message-delivery fail-closed behavior only
- Rejection codes and snapshot restoration errors must preserve current public enum values
- No new production error types or translation layers are introduced

## Test plan
1. Add a contract test referencing `runtime_guard_message_delivery.rs` before that file exists so the red phase is a real missing-surface failure.
2. Add a dedicated integration surface covering acceptance, replay rejection, nonce rejection, snapshot export/restore, and invalid snapshot failures via the public API.
3. Run targeted message-delivery contract and integration tests.
4. Run the full `kamn-runtime-guards` crate tests.
5. Run `test_file_size_policy` and refresh its baseline only if the new test targets change inventory counts.

## Phase 6 notes
- No production message-delivery behavior changes were required.
- Integration for this issue is the dedicated crate-level message-delivery surface plus the contract pin that prevents silent removal of that surface.
- Adding the two new test targets increased the workspace `test_file_total` baseline from `475` to `477`.
