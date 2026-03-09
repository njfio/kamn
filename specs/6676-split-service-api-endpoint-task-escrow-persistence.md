# Issue 6676: Split service_api_endpoint_tests Task and Escrow Persistence Tranche

## Objective

Extract the task and escrow persistence coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file continues shrinking and the task/escrow state flows become reviewable by concern.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- new task-escrow-persistence submodule declaration from the root test file
- moved task/escrow route and restart persistence coverage in bounded files
- updated split-contract coverage for the new extraction markers and a lower staged root threshold

## Boundaries/Non-goals

- Do not rewrite service API behavior
- Do not extract content lifecycle, bridge, relay, mailbox, or rate-limit coverage in this issue
- Do not modify existing extracted files except for necessary root-module wiring
- Keep the tranche limited to task and escrow persistence coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps moved task/escrow persistence tests after the extraction
- new submodules exceed the 200 LOC budget
- the staged root threshold does not improve after the extraction
- contract coverage is lost or detached from the real `kamn-node` test entrypoint
- extracted tests stop exercising the real task/escrow persistence paths

## Acceptance Criteria

- [x] AC-1: `service_api_endpoint_tests.rs` declares a new task-escrow-persistence submodule and no longer retains the moved task/escrow persistence test markers.
- [x] AC-2: Extracted task/escrow persistence files stay at or below 200 lines each.
- [x] AC-3: The staged root threshold ratchets down from `5718` to `5350` lines or lower.
- [x] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [x] AC-5: At least one extracted task/escrow persistence test passes from the real `kamn-node` test module path.

## Files To Touch

- `specs/6676-split-service-api-endpoint-task-escrow-persistence.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains moved task/escrow persistence tests or the extracted files exceed budget.
- The moved tests must continue exercising the real `kamn-node` task and escrow persistence paths.
- No silent fallback to duplicate coverage in both the root file and extracted modules.

## Test Plan

1. Add red split-contract assertions for the task-escrow-persistence module markers, moved test markers, and a lowered staged root threshold.
2. Extract the task and escrow persistence coverage into bounded files until the split contract passes.
3. Run the targeted split contract and directly affected `kamn-node` persistence tests.
4. Record integration evidence and any deviations in this spec.

## Phase 6 Evidence

- Root module wiring:
  - `#[path = "service_api_endpoint_tests/task_escrow_persistence_contract_tests.rs"]`
  - `mod task_escrow_persistence_contract_tests;`
- Final line counts:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`: `5331`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests.rs`: `6`
  - `.../task_escrow_routes_contract_tests.rs`: `44`
  - `.../task_escrow_restart_contract_tests.rs`: `51`
  - `.../support.rs`: `184`
- Verified commands:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_persists_task_and_escrow_state_across_restart -- --nocapture`

## Deviations

- No functional deviation from the issue scope. The only refactor after green was consolidating repeated state-file env setup into a shared helper.
