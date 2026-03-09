# 6645 - Split service_api_endpoint_tests.rs into bounded modules

## Objective

Reduce `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` by extracting a cohesive first wave of route/auth/scope coverage into dedicated submodules while preserving the existing test names, wiring, and split-contract enforcement. Land a staged file-size reduction that moves the root shell materially downward and establishes the repeatable pattern for follow-up waves.

## Inputs/Outputs

### Inputs
- Existing root test shell: `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- Existing extracted submodules:
  - `service_api_endpoint_tests/websocket_contract_tests.rs`
  - `service_api_endpoint_tests/balance_contract_tests.rs`
- Existing split contract: `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- Main-test module wiring in `crates/kamn-node/src/main_tests.rs`

### Outputs
- New extracted route/auth/scope-focused submodule(s) under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
- Updated root shell with those tests/helpers removed and submodules declared
- Updated split contract asserting ownership of moved markers and a staged root-file reduction target
- Preserved test names and runtime behavior under the real `kamn-node` main-test entrypoint

## Boundaries/Non-goals

- Do not rewrite service API behavior or route semantics
- Do not redesign all service API tests in one issue
- Do not rename test cases unless required by module visibility rules
- Do not attempt to bring the full root file under 200 LOC in this single wave if a staged threshold is needed

## Failure Modes

- Moved tests stop compiling because helpers/constants are not exported correctly
- Root shell retains moved tests/helpers and duplicates ownership
- Split contract markers drift from actual module contents
- Extracted module grows into another monolith rather than a cohesive concern slice
- Existing main-test wiring stops discovering or running the moved tests
- File-size staged threshold is not enforced after the move

## Acceptance Criteria

- [ ] Route/auth/scope policy tests are extracted from the root file into dedicated submodule(s) organized by concern
- [ ] The extracted test names remain runnable from the real `kamn-node` main-test wiring
- [ ] Shared helpers/constants are moved only when they are primarily owned by the extracted concern slice
- [ ] `service_api_endpoint_tests.rs` is reduced to an explicitly staged smaller threshold enforced by split contract tests
- [ ] The split contract asserts both module declaration and moved-marker ownership for the new submodule(s)
- [ ] No new extracted file exceeds the repo 200 LOC limit unless explicitly staged and contract-enforced in this issue

## Files To Touch

- `specs/6645-split-service-api-endpoint-tests.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- new files under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`
- `crates/kamn-node/src/main_tests.rs` only if module wiring needs adjustment
- file-size policy / extraction-budget tests only if the staged threshold contract requires it

## Error Semantics

- Existing test failure semantics remain unchanged; this is a structural extraction only
- Split contracts fail closed when moved markers remain in the root file or are absent from the destination module
- Size-budget/split-contract assertions fail closed on staged-threshold drift

## Test Plan

1. Red: extend `service_api_endpoint_tests_split_contract.rs` with route/auth/scope moved-marker assertions and a staged root-file cap
2. Red: add any module-visibility or compile-surface tests needed so the extracted modules must stay wired from `main_tests`
3. Green: move the route/auth/scope test cluster and only the helpers/constants it actually owns
4. Refactor: trim duplicated helper ownership and keep each new file under the repo size ceiling where possible
5. Integration: run the focused `kamn-node` split-contract and relevant `main_tests` coverage through the real test entrypoints

## Notes / Deviations

- This issue is a first extraction wave, not the full end-state decomposition of the 8.8K root file.
- The staged reduction target will be set from the current file after red contracts are written so the ratchet is evidence-backed rather than guessed.

## Integration Evidence

- `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture`
- `cargo test -p kamn-node auth_scope_contract_tests -- --nocapture`
