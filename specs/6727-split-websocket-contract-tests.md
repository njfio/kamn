# Objective

Split `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs` into bounded sibling modules that preserve the existing websocket contract coverage while reducing the websocket root shell under the active size policy.

# Inputs/Outputs

## Inputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs` at 1129 LOC on current `origin/main`
- Existing service API endpoint websocket coverage for upgrade flow, live-stream delivery, presence mode, and rejection paths
- Current `kamn-node` test entrypoint wiring for `websocket_contract_tests`
- Active touched-Rust size policy

## Outputs
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs` reduced to a bounded root shell
- New bounded sibling modules under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests/` grouped by websocket concern
- Contract coverage that fails if the websocket root shell regresses or the extracted module layout disappears
- Updated spec evidence showing the extracted websocket contract surface still runs through the real `kamn-node` test entrypoint and passes touched-Rust size checks

# Boundaries/Non-goals

- Do not change service API websocket runtime behavior
- Do not redesign unrelated service API endpoint test domains
- Do not weaken websocket coverage or assertions to satisfy file-size policy
- Do not add new dependencies

# Failure modes

- `websocket_contract_tests.rs` remains an oversized monolith
- extracted websocket modules are arbitrary slices rather than matching the existing websocket concern seams
- websocket upgrade, presence-mode, or rejection coverage is lost during extraction
- the root contract no longer enforces the websocket module layout or root shell budget
- touched-Rust size policy fails on the issue write set

# Acceptance criteria (testable booleans)

- [x] `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs` is reduced to a bounded root shell under the active size policy
- [x] websocket contract coverage is split into coherent sibling modules that reflect the existing websocket concern seams
- [x] extracted files added by this issue remain within the active touched-Rust size policy on the issue write set
- [x] a contract fails if the websocket root shell regresses or the extracted module layout disappears
- [x] `cargo test -p kamn-node websocket_contract_tests -- --nocapture` passes after extraction
- [x] touched-Rust size policy returns `policy_decision=GO` on the issue write set

# Files to touch

- `specs/6727-split-websocket-contract-tests.md`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs`
- new files under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests/`
- `crates/kamn-node/tests/` websocket extraction contract file(s) as needed

# Error semantics

- Preserve the current hard-fail websocket assertions for upgrade handling, presence-mode behavior, event delivery, and rejection paths
- New extraction contracts fail with exact missing-path, missing-module-marker, or root-budget details
- No fallback path to inline websocket helper bodies or partial websocket coverage

# Test plan

1. Add a red extraction contract requiring a bounded websocket root shell and a concern-based `websocket_contract_tests/` layout.
2. Extract the websocket contract tests into sibling modules grouped by upgrade flow, live events, presence mode, and rejection paths.
3. Run `cargo test -p kamn-node websocket_contract_tests -- --nocapture`.
4. Run the websocket extraction contract.
5. Run the touched-Rust size policy on the issue write set.
6. Record final evidence and any deviations in this spec before opening the PR.

# Final evidence

- `cargo test -p kamn-node websocket_contract_tests -- --nocapture`
  - result: `18 passed, 0 failed`
- `cargo test -p kamn-node --test websocket_contract_tests_extraction_contract -- --nocapture`
  - result: `3 passed, 0 failed`
- `bash scripts/ci/check_touched_rust_size_policy.sh --output-json /home/n/Code/kamn/tmp/6727-touched-size.json`
  - result: `status=pass`, `policy_decision=GO`

# Measured outcome

- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/websocket_contract_tests.rs`
  - reduced from `1129` LOC to a bounded root shell
- extracted websocket modules now cover:
  - upgrade flow and live delivery
  - presence projection and validation
  - legacy-header rejection coverage
  - route/header rejection coverage
  - shared websocket request/frame support

# Deviations

- A broad `cargo fmt --all` run temporarily touched unrelated files during Phase 5. Those changes were restored before final verification, and the final touched-Rust report was rerun on the websocket-only write set.
