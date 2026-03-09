# Issue 6682: Split service_api_endpoint_tests Mailbox Relay Delivery Tranche

## Objective

Extract the mailbox and relay delivery coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file continues shrinking and the recipient-delivery path is reviewable by concern.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- a new mailbox-relay-delivery submodule declaration from the root test file
- moved mailbox, relay delivery, relay DID rejection, relayed-to-delivered, and durable spool coverage in bounded files
- updated split-contract coverage for the new extraction markers and a lower staged root threshold

## Boundaries/Non-goals

- Do not rewrite service API behavior
- Do not extract rate-limit, concurrency, or lifecycle projection coverage in this issue
- Do not modify existing extracted files except for necessary root-module wiring
- Keep the tranche limited to mailbox and relay delivery coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps the moved mailbox or relay delivery test markers after extraction
- new submodules exceed the 200 LOC budget
- the staged root threshold does not improve after extraction
- contract coverage is lost or detached from the real `kamn-node` test entrypoint
- extracted tests stop exercising the real mailbox and relay delivery path

## Acceptance Criteria

- [x] AC-1: `service_api_endpoint_tests.rs` declares a new mailbox-relay-delivery submodule and no longer retains the moved mailbox and relay delivery test markers.
- [x] AC-2: Extracted mailbox-relay-delivery files stay at or below 200 lines each.
- [x] AC-3: The staged root threshold ratchets down from `4856` to `4525` lines or lower.
- [x] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [x] AC-5: At least one extracted mailbox or relay delivery test passes from the real `kamn-node` test module path.

## Files To Touch

- `specs/6682-split-service-api-endpoint-mailbox-relay-delivery.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains the moved mailbox or relay delivery coverage or the extracted files exceed budget.
- The moved tests must continue exercising the real `kamn-node` mailbox and relay delivery paths.
- No silent fallback to duplicate coverage in both the root file and extracted modules.

## Test Plan

1. Add red split-contract assertions for the mailbox-relay-delivery module markers, moved test markers, and a lowered staged root threshold.
2. Extract the mailbox and relay delivery coverage into bounded files until the split contract passes.
3. Run the targeted split contract and directly affected `kamn-node` mailbox or relay delivery tests.
4. Record integration evidence and any deviations in this spec.

## Deviations

- The extracted cross-node relay leaf no longer duplicates the sender spool append/idempotency assertion that was previously embedded in the root file variant. Durable spool coverage remains exercised by the dedicated relay spool test.
- The extracted cross-node relay leaf no longer includes the restart re-query tail. Restart and relayed-to-delivered persistence coverage remains in `relay_status_contract_tests.rs`.
- Relay-status fixture generation now uses `serde_json::json!` plus `serde_json::to_string_pretty(...)` instead of hand-built JSON strings because the string-built fixture was malformed and prevented the service API server from starting during the moved tests.

## Phase 6 Evidence

- Root wiring:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` declares `#[path = "service_api_endpoint_tests/mailbox_relay_delivery_contract_tests.rs"]` and `mod mailbox_relay_delivery_contract_tests;`
- File sizes:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`: `3248`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests.rs`: `12`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/recipient_mailbox_contract_tests.rs`: `148`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_delivery_contract_tests.rs`: `160`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_did_rejection_contract_tests.rs`: `117`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/relay_status_contract_tests.rs`: `68`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/support.rs`: `180`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/mailbox_relay_delivery_contract_tests/state_support.rs`: `57`
- Touched-Rust size policy:
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6682-touched-size.json`
  - Result: `status=pass`, `policy_decision=GO`
- Targeted evidence:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_recipient_mailbox_and_delivery_status_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_cross_node_relay_delivery_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_recipient_query_promotes_relayed_to_delivered -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node regression_service_api_endpoint_non_recipient_query_keeps_relayed_status_across_restart -- --nocapture`
