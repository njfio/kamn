# 6806 Split tcp transport adapter tests

## Objective
Split `crates/kamn-sdk/tests/tcp_transport_adapter.rs` into a thin root shell plus bounded concern-based modules while preserving the current TCP transport adapter coverage.

## Inputs/Outputs
- Input: the current `tcp_transport_adapter.rs` monolithic test target on `main`
- Output: a root shell that wires bounded sibling modules for envelope crypto/validation, relay flow, reconnect/replay guard coverage, tamper/forgery rejection, and local performance coverage

## Boundaries/Non-goals
- Do not change production TCP transport adapter behavior
- Do not add new dependencies or features
- Do not weaken replay, tamper, DID-binding, or performance assertions
- Do not change public APIs

## Failure modes
- Root shell remains above the staged root budget
- Extracted files exceed the 200 LOC policy
- Shared DID/signer helpers drift from the original cryptographic behavior
- Replay/reconnect coverage loses deterministic nonce-guard assertions
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-sdk/tests/tcp_transport_adapter.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for envelope validation, relay flow, replay/reconnect coverage, tamper/forgery rejection, and performance coverage
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-sdk --test tcp_transport_adapter_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-sdk --test tcp_transport_adapter -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6806-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6806-split-tcp-transport-adapter.md`
- `crates/kamn-sdk/tests/tcp_transport_adapter.rs`
- `crates/kamn-sdk/tests/tcp_transport_adapter_extraction_contract.rs`
- `crates/kamn-sdk/tests/tcp_transport_adapter/**`

## Error semantics
- Tests remain fail-closed and preserve current deterministic signature/DID-binding/replay reason assertions
- Shared helpers may panic in tests with explicit messages when network setup fails
- No silent fallbacks or swallowed transport errors

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the test target into bounded sibling modules and shared support
3. Run the extraction contract target
4. Run the real `tcp_transport_adapter` target
5. Run the touched-Rust size checker against `origin/main`

## Phase 6 evidence
- `cargo test -p kamn-sdk --test tcp_transport_adapter_extraction_contract -- --nocapture`
- `cargo test -p kamn-sdk --test tcp_transport_adapter -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6806-touched-size-refactor.json`
- Final touched-Rust result: `policy_decision=GO`

## Deviations
- None.
