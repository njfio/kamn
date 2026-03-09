# 6690 Split service_api_endpoint_tests Residual Root Tests

## Objective

Extract the remaining residual root tests from `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file falls to or below the 200-line limit while preserving the shared helper surface and the real service API test coverage.

## Inputs/Outputs

- Inputs:
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
  - `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
  - shared helper modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/shared_support/`
- Outputs:
  - residual test submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
  - a reduced root file with only module declarations, shared-support glue, and minimal imports
  - updated split-contract coverage for the new residual-test split and lower root budget

## Boundaries/Non-goals

- Do not change service API runtime behavior or assertion semantics
- Do not rework shared helper modules beyond import adjustments required by the move
- Do not weaken the residual tests to satisfy the split
- Do not rewrite previously extracted tranches except for necessary module-path adjustments

## Failure Modes

- Residual tests lose access to the shared helper surface after extraction
- The root file remains above 200 LOC
- New extracted residual-test files exceed 200 LOC
- Split contract still passes without actually proving the residual tests moved
- Extracted residual tests stop exercising real service API paths

## Acceptance Criteria

- [ ] Extract the remaining residual root tests from `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into dedicated bounded modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
- [ ] Reduce `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` to `<= 200` LOC
- [ ] Keep each new extracted file at `<= 200` LOC
- [ ] Update `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs` to assert the residual-test split and ratchet the root threshold downward to the actual target
- [ ] `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes
- [ ] At least one extracted residual test passes using the real shared helper surface
- [ ] At least one previously extracted tranche test still passes using the same shared helper surface

## Files To Touch

- `specs/6690-split-service-api-endpoint-residual-root-tests.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- new residual-test modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
- existing modules only as needed for import/module-path updates

## Error Semantics

- Tests remain fail-closed and keep the current panic/assert behavior on invalid setup or unexpected responses
- Helper-backed request/error parsing semantics must remain unchanged
- No silent fallbacks or assertion weakening are allowed

## Test Plan

1. Add a red split contract that requires new residual-test submodules, removes residual-test markers from the root file, and ratchets the root threshold to `<= 200` LOC.
2. Run `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` and confirm failure before implementation.
3. Extract the residual tests into bounded submodules and reduce the root file to module glue.
4. Re-run:
   - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
   - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node unit_service_api_endpoint_serde_payload_roundtrip_contracts -- --nocapture`
   - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --nocapture`
5. Run `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6690-touched-size.json` on the final write set.
