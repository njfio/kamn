# 6802 Split p2p libp2p native adapter runtime

## Objective
Split `crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime.rs` into a thin root shell plus bounded concern-based modules while preserving the existing libp2p native adapter runtime coverage.

## Inputs/Outputs
- Input: the current `p2p_libp2p_native_adapter_runtime.rs` monolithic integration test target on `main`
- Output: a root shell that wires bounded sibling modules for config validation, runtime/backend markers, discovery and gossip socket flows, partition/reason-code coverage, and local-heavy performance coverage

## Boundaries/Non-goals
- Do not change production libp2p native adapter behavior
- Do not add new dependencies or features
- Do not weaken discovery, gossip, partition, or performance assertions
- Do not change public APIs

## Failure modes
- Root shell remains above the staged line budget
- Extracted files exceed the 200 LOC policy
- Runtime socket/discovery helpers lose deterministic retry or timeout behavior after extraction
- Partition/reason-code assertions drift from the current fail-closed behavior
- Extraction contract markers drift from the real root layout

## Acceptance criteria
- [ ] `crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime.rs` is reduced to a thin root shell at or below 180 LOC
- [ ] Root shell wires bounded sibling modules for config validation, marker/runtime behavior, discovery/gossip flows, partition/reason-code coverage, and performance coverage
- [ ] All extracted files touched by the split remain at or below 200 LOC
- [ ] `cargo test -p kamn-core --test p2p_libp2p_native_adapter_runtime_extraction_contract -- --nocapture` passes
- [ ] `cargo test -p kamn-core --test p2p_libp2p_native_adapter_runtime -- --nocapture` passes
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6784-remote --base-ref origin/main --output-json /tmp/6802-touched-size.json` returns `policy_decision=GO`

## Files to touch
- `specs/6802-split-p2p-libp2p-native-adapter-runtime.md`
- `crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime.rs`
- `crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime_extraction_contract.rs`
- `crates/kamn-core/tests/p2p_libp2p_native_adapter_runtime/**`

## Error semantics
- Tests remain fail-closed and preserve current deterministic reason-code assertions
- Shared support helpers may panic in tests with explicit messages when socket setup or drain expectations fail
- No silent fallbacks or swallowed transport errors

## Test plan
1. Add an extraction contract that fails while the root file is still monolithic
2. Split the runtime target into bounded sibling modules and shared support
3. Run the extraction contract target
4. Run the real `p2p_libp2p_native_adapter_runtime` target
5. Run the touched-Rust size checker against `origin/main`
