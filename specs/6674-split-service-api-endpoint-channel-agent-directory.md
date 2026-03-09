# Issue 6674: Split service_api_endpoint_tests Channel and Agent Directory Tranche

## Objective

Extract the channel and agent directory contract coverage out of `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs` into bounded submodules so the root file continues shrinking and the directory-state flows become reviewable by concern.

## Inputs/Outputs

### Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- existing extracted submodules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`

### Outputs
- new channel-agent-directory submodule declaration from the root test file
- moved channel message listing, channel restart persistence, agent profile restart persistence, and agent metadata directory contract coverage in bounded files
- updated split-contract coverage for the new extraction markers and a lower staged root threshold

## Boundaries/Non-goals

- Do not rewrite service API behavior
- Do not extract task, escrow, content, bridge, relay, or rate-limit coverage in this issue
- Do not modify existing websocket, auth/scope, route-render, or message-persistence extracted files except for necessary root-module wiring
- Keep the tranche limited to channel/agent directory-state coverage

## Failure Modes

- `service_api_endpoint_tests.rs` keeps moved channel/agent directory tests after the extraction
- new submodules exceed the 200 LOC budget
- the staged root threshold does not improve after the extraction
- contract coverage is lost or detached from the real `kamn-node` test entrypoint
- extracted tests stop exercising the real channel/agent directory persistence and search paths

## Acceptance Criteria

- [ ] AC-1: `service_api_endpoint_tests.rs` declares a new channel-agent-directory submodule and no longer retains the moved channel/agent directory test markers.
- [ ] AC-2: Extracted channel-agent-directory files stay at or below 200 lines each.
- [ ] AC-3: The staged root threshold ratchets down from `6571` to `5900` lines or lower.
- [ ] AC-4: `cargo test -p kamn-node service_api_endpoint_tests_split_contract -- --nocapture` passes.
- [ ] AC-5: At least one extracted channel/agent directory test passes from the real `kamn-node` test module path.

## Files To Touch

- `specs/6674-split-service-api-endpoint-channel-agent-directory.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests_split_contract.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/channel_agent_directory_contract_tests/*.rs`

## Error Semantics

- Split contracts fail closed when the root file retains moved channel/agent directory tests or the extracted files exceed budget.
- The moved tests must continue exercising the real `kamn-node` channel and agent directory paths.
- No silent fallback to duplicate coverage in both the root file and extracted modules.

## Test Plan

1. Add red split-contract assertions for the channel-agent-directory module markers, moved test markers, and a lowered staged root threshold.
2. Extract the channel and agent directory coverage into bounded files until the split contract passes.
3. Run the targeted split contract and directly affected `kamn-node` directory-state tests.
4. Record integration evidence and any deviations in this spec.
