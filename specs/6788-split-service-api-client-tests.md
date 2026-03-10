# Objective

Extract the oversized test surface from `crates/kamn-sdk/tests/service_api_client.rs` into bounded sibling modules while preserving the existing `kamn-sdk` service API client contract coverage and real runtime wiring.

# Inputs/Outputs

Inputs:
- `crates/kamn-sdk/tests/service_api_client.rs`
- embedded TLS/server/auth/request parsing helpers in the current root file
- the real `kamn-sdk` service API client test target and its route/websocket coverage

Outputs:
- bounded module tree under `crates/kamn-sdk/tests/service_api_client/`
- extraction contract covering the new module layout and staged root budget
- reduced root `service_api_client.rs`

# Boundaries/Non-goals

- Do not change production `kamn-sdk` service API client behavior or request semantics.
- Do not rewrite unrelated `kamn-sdk` tests or helpers outside this target.
- Do not weaken TLS, auth, CRLF, websocket, or route-contract assertions during extraction.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted sibling files exceed the active file-size budget
- helper moves break the embedded HTTPS or websocket test server harness
- route/auth validation assertions disappear or drift during extraction
- touched-Rust ratchet fails on newly oversized touched files or functions

# Acceptance criteria

- [x] root test surface is extracted from `crates/kamn-sdk/tests/service_api_client.rs` into bounded sibling modules organized by concern
- [x] root `service_api_client.rs` is reduced below a staged extraction cap enforced by a new extraction contract
- [x] extracted sibling files stay within the active file-size budget
- [x] the real `kamn-sdk` service API client target remains wired and passes
- [x] `cargo test -p kamn-sdk --test service_api_client -- --nocapture` passes
- [x] the extraction contract passes
- [x] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-sdk/tests/service_api_client.rs`
- `crates/kamn-sdk/tests/service_api_client/**`
- `crates/kamn-sdk/tests/*extraction_contract*.rs`
- `specs/6788-split-service-api-client-tests.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module, marker, or budget diagnostics.
- Existing service API client failures remain ordinary Rust assertion failures with no silent fallbacks.
- Embedded HTTPS/websocket harness setup failures remain hard-fail test errors.

# Test plan

1. Add a red extraction contract asserting the new module layout and a staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the root file into bounded sibling modules and nested support modules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-sdk --test service_api_client -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6788-touched-size.json`.

# Planned module seams

- `support.rs` for shared env, TLS server, request parsing, and auth helpers
- `tls_contract_tests.rs` for HTTPS success/failure coverage
- `input_validation_contract_tests.rs` for CRLF, DID, and scope validation regressions
- `signed_http_route_contract_tests.rs` for signed route execution coverage
- `websocket_contract_tests.rs` for websocket frame and extended-length coverage
- `route_family_contract_tests.rs` for channel, registration, task/escrow, and bridge route groups

# Results

- `crates/kamn-sdk/tests/service_api_client.rs` now acts as a 12 LOC root shell.
- Shared helpers are split under `crates/kamn-sdk/tests/service_api_client/support/`.
- Route-family and signed-route coverage are further subdivided so touched files and functions stay under the active ratchet.

# Verification evidence

- `cargo test -p kamn-sdk --test service_api_client_extraction_contract -- --nocapture`
- `cargo test -p kamn-sdk --test service_api_client -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6788-touched-size-refactor.json`
- touched-Rust result: `policy_decision=GO`

# Deviations

- None.
