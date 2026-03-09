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

- [ ] AC-1: `service_api_endpoint_tests.rs` declares a new content-lifecycle-restart submodule and no longer retains the moved content lifecycle restart test marker.
- [ ] AC-2: Extracted content-lifecycle-restart files stay at or below 200 lines each.
- [ ] AC-3: The staged root threshold ratchets down from `5331` to `5100` lines or lower.
- [ ] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [ ] AC-5: At least one extracted content lifecycle restart test passes from the real `kamn-node` test module path.

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
