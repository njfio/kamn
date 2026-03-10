# Objective

Extract the oversized test surface from `crates/kamn-sdk/tests/live_transport_agent.rs` into bounded sibling modules while preserving the real `kamn-sdk` live transport agent contract coverage and runtime wiring.

# Inputs/Outputs

Inputs:
- `crates/kamn-sdk/tests/live_transport_agent.rs`
- embedded env/auth/http server helpers currently in the root test file
- the real `kamn-sdk` live transport agent test target

Outputs:
- bounded module tree under `crates/kamn-sdk/tests/live_transport_agent/`
- extraction contract covering the staged root budget and required module layout
- reduced root `live_transport_agent.rs`

# Boundaries/Non-goals

- Do not change production `kamn-sdk` live transport behavior or request semantics.
- Do not rewrite unrelated SDK integration tests.
- Do not weaken message, resolve, register, search, channel, TLS/auth, or transport-mode assertions during extraction.

# Failure modes

- root file still exceeds the staged extraction cap
- extracted sibling files exceed the active file-size budget
- helper moves break the embedded HTTP server harness or auth validation path
- route/transport assertions disappear or drift during extraction
- touched-Rust ratchet fails on newly oversized touched files or functions

# Acceptance criteria

- [ ] root test surface is extracted from `crates/kamn-sdk/tests/live_transport_agent.rs` into bounded sibling modules organized by concern
- [ ] root `live_transport_agent.rs` is reduced below a staged extraction cap enforced by a new extraction contract
- [ ] extracted sibling files stay within the active file-size budget
- [ ] the real `kamn-sdk` live transport agent target remains wired and passes
- [ ] `cargo test -p kamn-sdk --test live_transport_agent -- --nocapture` passes
- [ ] the extraction contract passes
- [ ] touched-Rust size policy returns `policy_decision=GO` for the staged write set

# Files to touch

- `crates/kamn-sdk/tests/live_transport_agent.rs`
- `crates/kamn-sdk/tests/live_transport_agent/**`
- `crates/kamn-sdk/tests/*extraction_contract*.rs`
- `specs/6790-split-live-transport-agent.md`

# Error semantics

- Extraction contract failures must hard-fail with explicit missing module, marker, or budget diagnostics.
- Existing live transport agent failures remain ordinary Rust assertion failures with no silent fallbacks.
- Embedded contract-server/auth/parser failures remain hard-fail test errors.

# Test plan

1. Add a red extraction contract asserting the module layout and staged root budget.
2. Run the extraction contract and confirm it fails on current `main`.
3. Extract the root file into bounded sibling modules and nested support modules where needed to stay under the 200 LOC cap.
4. Run `cargo test -p kamn-sdk --test live_transport_agent -- --nocapture`.
5. Run the extraction contract again and confirm green.
6. Run `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6790-touched-size.json`.

# Planned module seams

- `support.rs` for env, auth, parsing, server, and deterministic helper utilities
- `transport_mode_contract_tests.rs` for config/mode and unsupported-path checks
- `message_contract_tests.rs` for send/status/alias/json-escape flows
- `resolve_profile_contract_tests.rs` for resolve/reputation/profile flows
- `registration_search_contract_tests.rs` for register/search route coverage
- `channel_contract_tests.rs` for channel creation and empty-id regression coverage
