# Issue 6678: Split service_api_endpoint_tests Content Lifecycle Restart Tranche

## Objective

Extract the content lifecycle restart coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file continues shrinking and the content-state restart flow becomes reviewable by concern.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- new content-lifecycle-restart submodule declaration from the root test file
- moved content register/expire/query/tombstone restart coverage in bounded files
- updated split-contract coverage for the new extraction markers and a lower staged root threshold

## Boundaries/Non-goals

- Do not rewrite service API behavior
- Do not extract bridge, relay, mailbox, or rate-limit coverage in this issue
- Do not modify existing extracted files except for necessary root-module wiring
- Keep the tranche limited to content lifecycle restart persistence coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps the moved content lifecycle restart test after extraction
- new submodules exceed the 200 LOC budget
- the staged root threshold does not improve after extraction
- contract coverage is lost or detached from the real `kamn-node` test entrypoint
- extracted tests stop exercising the real content lifecycle restart path

## Acceptance Criteria

- [x] AC-1: `service_api_endpoint_tests.rs` declares a new content-lifecycle-restart submodule and no longer retains the moved content lifecycle restart test marker.
- [x] AC-2: Extracted content-lifecycle-restart files stay at or below 200 lines each.
- [x] AC-3: The staged root threshold ratchets down from `5331` to `5100` lines or lower.
- [x] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [x] AC-5: At least one extracted content lifecycle restart test passes from the real `kamn-node` test module path.

## Files To Touch

- `specs/6678-split-service-api-endpoint-content-lifecycle-restart.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/content_lifecycle_restart_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/content_lifecycle_restart_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains the moved content lifecycle restart test or the extracted files exceed budget.
- The moved test must continue exercising the real `kamn-node` content lifecycle restart path.
- No silent fallback to duplicate coverage in both the root file and extracted modules.

## Test Plan

1. Add red split-contract assertions for the content-lifecycle-restart module markers, moved test marker, and a lowered staged root threshold.
2. Extract the content lifecycle restart coverage into bounded files until the split contract passes.
3. Run the targeted split contract and directly affected `kamn-node` restart test.
4. Record integration evidence and any deviations in this spec.

## Phase 6 Evidence

- Root module wiring:
  - `#[path = "service_api_endpoint_tests/content_lifecycle_restart_contract_tests.rs"]`
  - `mod content_lifecycle_restart_contract_tests;`
- Final line counts:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`: `5071`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/content_lifecycle_restart_contract_tests.rs`: `4`
  - `.../content_lifecycle_restart_contract_tests.rs`: `82`
  - `.../support.rs`: `195`
- Verified commands:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_persists_content_lifecycle_state_across_restart -- --nocapture`

## Deviations

- No behavioral deviation from scope. The refactor split the restart assertions into small local helpers to satisfy the function-size discipline while keeping the extracted support file under the 200-line budget.
