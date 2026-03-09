# Issue 6672: Split service_api_endpoint_tests Message Persistence Tranche

## Objective

Extract the message-persistence contract coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file continues shrinking and the restart/runtime-evidence message flows become reviewable by concern.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- new message-persistence submodule declaration from the root test file
- moved message restart and data-layer runtime evidence contract coverage in bounded files
- updated split-contract coverage for the new extraction markers and a lowered staged root threshold

## Boundaries/Non-goals

- Do not rewrite service API behavior
- Do not extract channel persistence, agent profile, or registration coverage in this issue
- Do not modify existing websocket, auth/scope, or route-render extracted files except for necessary root-module wiring
- Keep the tranche limited to message restart persistence and send-path runtime evidence coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps moved message-persistence tests after the extraction
- new submodules exceed the 200 LOC budget
- the staged root threshold does not improve after the extraction
- contract coverage is lost or detached from the real `kamn-node` test entrypoint
- extracted tests stop exercising the real message send/query persistence path

## Acceptance Criteria

- [x] AC-1: `service_api_endpoint_tests.rs` declares a new message-persistence submodule and no longer retains the moved message-persistence test markers.
- [x] AC-2: Extracted message-persistence files stay at or below 200 lines each.
- [x] AC-3: The staged root threshold ratchets down from `6923` to `6650` lines or lower.
- [x] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [x] AC-5: At least one extracted message-persistence test passes from the real `kamn-node` test module path.

## Files To Touch

- `specs/6672-split-service-api-endpoint-message-persistence.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains moved message-persistence tests or the extracted files exceed budget.
- The moved tests must continue exercising the real `kamn-node` service API persistence path.
- No silent fallback to duplicate coverage in both the root file and extracted modules.

## Test Plan

1. Add red split-contract assertions for the message-persistence module markers, moved test markers, and a lowered staged root threshold.
2. Extract the message-persistence coverage into bounded files until the split contract passes.
3. Run the targeted split contract and directly affected `kamn-node` message-persistence tests.
4. Record integration evidence and any deviations in this spec.

## Phase 6 Evidence

- Root module wiring:
  - `#[path = "service_api_endpoint_tests/message_persistence_contract_tests.rs"]`
  - `mod message_persistence_contract_tests;`
- Final line counts:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`: `6571`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests.rs`: `6`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/message_restart_contract_tests.rs`: `46`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/message_runtime_evidence_contract_tests.rs`: `62`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/message_persistence_contract_tests/support.rs`: `156`
- Verified commands:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_persists_message_state_across_restart -- --nocapture`

## Deviations

- The extraction uses a private `support.rs` helper module to keep each new file within the 200 LOC limit while preserving real-path message send/query coverage.
