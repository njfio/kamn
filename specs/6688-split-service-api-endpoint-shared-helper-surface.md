# 6688 Split service_api_endpoint_tests Shared Helper Surface

## Objective

Extract the remaining shared helper surface out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` so the root test file keeps only residual test entrypoints, imports, and minimal module glue while preserving the existing helper-backed behavior used by the already-split submodules.

## Inputs/Outputs

- Inputs:
  - Existing root helper constants, structs, and helper functions in `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
  - Existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
  - Existing split contract in `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- Outputs:
  - Dedicated bounded helper/support modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
  - Root-file glue that re-exports or imports the extracted helpers for existing submodules
  - Updated split contract ratcheting the staged root threshold downward again

## Boundaries/Non-goals

- Do not change service API behavior or residual test semantics
- Do not rewrite already-extracted test tranches beyond import adjustments needed for helper relocation
- Do not weaken helper-backed assertions to make the split pass
- Do not change production code or service API route/auth behavior

## Failure Modes

- Extracted submodules can no longer resolve shared helper symbols through the root namespace
- Helper extraction leaves the root file above the staged threshold
- A new helper file exceeds 200 LOC
- Residual root-path tests no longer pass because env, transport, or parsing helpers moved incorrectly
- TLS/http helper relocation breaks live request coverage or response parsing assumptions

## Acceptance Criteria

- [ ] Shared helper constants, structs, and helper functions are extracted from `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into dedicated modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
- [ ] The root file keeps only module declarations, minimal glue, and the residual root tests
- [ ] Each new extracted file is at or below 200 LOC
- [ ] `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs` ratchets the root threshold downward from 1350 LOC
- [ ] `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes
- [ ] At least one residual root-path test passes using the extracted helper surface

## Files To Touch

- `specs/6688-split-service-api-endpoint-shared-helper-surface.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- New helper modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
- Existing extracted test modules only as needed for import/module-path adjustments

## Error Semantics

- Test helpers remain fail-closed and panic on invalid local test setup or transport failures
- Extraction must preserve existing error payload parsing and assertion semantics
- No silent fallback paths or weakened helper behavior are allowed

## Test Plan

1. Add a red split contract that requires the new helper modules, removes moved helper markers from the root file, enforces new file budgets, and ratchets the root threshold downward.
2. Run `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` and confirm failure before implementation.
3. Extract the helper surface into bounded support modules and wire the root namespace so existing submodules still compile.
4. Re-run the split contract and at least one residual root-path test:
   - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
   - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node unit_service_api_endpoint_serde_payload_roundtrip_contracts -- --nocapture`
5. Run touched Rust size policy on the extracted set.

## Phase 6 Evidence

- Root helper surface now wires through `service_api_endpoint_tests/shared_support.rs` and its bounded support modules.
- Residual root test path remains live through the extracted helpers:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node unit_service_api_endpoint_serde_payload_roundtrip_contracts -- --nocapture`
- Extracted transport integration path remains live through the shared helper surface:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node integration_service_api_endpoint_tls_mode_serves_required_https_routes -- --nocapture`
- Split contract still passes after the helper extraction:
  - `CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
- Touched Rust size policy passes for the full `#6688` write set:
  - `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /tmp/6688-touched-size.json`

## Measured Outcome

- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`: `1224 -> 381` lines
- Added bounded helper modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/shared_support/`
