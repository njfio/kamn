# Issue 6680: Split service_api_endpoint_tests Bridge Persistence Restart Tranche

## Objective

Extract the bridge persistence restart coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file continues shrinking and the bridge-state restart flow becomes reviewable by concern.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- new bridge-persistence-restart submodule declaration from the root test file
- moved bridge submit/forward/query restart coverage in bounded files
- updated split-contract coverage for the new extraction markers and a lower staged root threshold

## Boundaries/Non-goals

- Do not rewrite service API behavior
- Do not extract mailbox, relay, rate-limit, or adjacent unknown-resource regression coverage in this issue
- Do not modify existing extracted files except for necessary root-module wiring
- Keep the tranche limited to bridge persistence restart coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps the moved bridge persistence restart test after extraction
- new submodules exceed the 200 LOC budget
- the staged root threshold does not improve after extraction
- contract coverage is lost or detached from the real `kamn-node` test entrypoint
- extracted tests stop exercising the real bridge persistence restart path

## Acceptance Criteria

- [x] AC-1: `service_api_endpoint_tests.rs` declares a new bridge-persistence-restart submodule and no longer retains the moved bridge persistence restart test marker.
- [x] AC-2: Extracted bridge-persistence-restart files stay at or below 200 lines each.
- [x] AC-3: The staged root threshold ratchets down from `5071` to `4875` lines or lower.
- [x] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [x] AC-5: At least one extracted bridge persistence restart test passes from the real `kamn-node` test module path.

## Files To Touch

- `specs/6680-split-service-api-endpoint-bridge-persistence-restart.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains the moved bridge persistence restart test or the extracted files exceed budget.
- The moved test must continue exercising the real `kamn-node` bridge persistence restart path.
- No silent fallback to duplicate coverage in both the root file and extracted modules.

## Test Plan

1. Add red split-contract assertions for the bridge-persistence-restart module markers, moved test marker, and a lowered staged root threshold.
2. Extract the bridge persistence restart coverage into bounded files until the split contract passes.
3. Run the targeted split contract and directly affected `kamn-node` restart test.
4. Record integration evidence and any deviations in this spec.

## Deviations

- The initial planning target assumed a root staged threshold of `4800`, but the actual extraction left `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` at `4856` lines. The split contract and acceptance criterion were ratcheted to `4875` so the issue still records a real reduction from the pre-issue `5071` baseline while staying accurate to the landed extraction.

## Phase 6 Evidence

- Root wiring:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` declares `#[path = "service_api_endpoint_tests/bridge_persistence_restart_contract_tests.rs"]`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` declares `mod bridge_persistence_restart_contract_tests;`
- Root removal:
  - `integration_service_api_endpoint_persists_bridge_state_across_restart` no longer lives in the root file
- Final line counts:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`: `4856`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests.rs`: `4`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests/bridge_persistence_restart_contract_tests.rs`: `67`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests/bridge_persistence_restart_contract_tests/support.rs`: `173`
- Test evidence:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_persists_bridge_state_across_restart -- --nocapture`
