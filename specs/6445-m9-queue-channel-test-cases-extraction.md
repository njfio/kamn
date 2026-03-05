# Spec: Issue 6445 - Extract M9 queue/channel test cases

## Objective
Extract queue and channel scenarios (`spec_c06` through `spec_c09`) from `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs` into a dedicated `queue_channel_cases` module while preserving root entrypoint names and behavior.

## Inputs/Outputs
- Inputs:
  - Existing inline C06-C09 tests in `data_layer_m9_realtime_delivery.rs`.
  - Existing split-contract guard `data_layer_m9_realtime_delivery_split_contract.rs`.
- Outputs:
  - New `crates/kamn-core/tests/data_layer_m9_realtime_delivery/queue_channel_cases.rs` containing C06-C09 scenario bodies.
  - Root wrappers for C06-C09 delegating to `queue_channel_cases`.
  - Split-contract markers for C06-C09 delegation and ownership.

## Boundaries/Non-goals
- No behavior changes to queue snapshot ordering, deferred queue ordering, duplicate-message rejection, or channel-membership authorization semantics.
- No production source changes under `crates/kamn-core/src`.
- No dependency additions.

## Failure modes
- Root file still owns inline C06-C09 logic.
- Split-contract does not enforce C06-C09 extraction markers.
- Assertion behavior changes during extraction.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `queue_channel_cases.rs` exists and contains C06-C09 scenario bodies.
- [ ] AC-2: root `data_layer_m9_realtime_delivery.rs` retains C06-C09 entrypoints and delegates to `queue_channel_cases` functions.
- [ ] AC-3: split-contract test enforces C06-C09 delegation/ownership markers.
- [ ] AC-4: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract` passes.
- [ ] AC-5: `cargo test -p kamn-core --test data_layer_m9_realtime_delivery` passes.

## Files to touch
- `specs/6445-m9-queue-channel-test-cases-extraction.md`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery/queue_channel_cases.rs` (new)
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery_split_contract.rs`

## Error semantics
- Preserve existing fail-closed behavior and reason-code assertions for C06-C09.

## Test plan
- Red:
  - Add split-contract markers for C06-C09 extraction before wiring module; verify failing test.
- Green:
  - Move C06-C09 bodies into `queue_channel_cases.rs` and delegate root wrappers.
- Refactor:
  - Deduplicate repeated literals in extracted module without behavior changes.
- Integration:
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_split_contract`
  - `cargo test -p kamn-core --test data_layer_m9_realtime_delivery`

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
