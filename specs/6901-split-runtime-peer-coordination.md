# 6901-split-runtime-peer-coordination

## Objective
Split `crates/kamn-core/src/runtime_peer_coordination.rs` into bounded concern-based modules while preserving existing runtime peer lifecycle, bounded queue/backpressure, authenticated peer-frame, deterministic proposal planner, and runtime wiring/transport profile behavior.

## Inputs/Outputs
- Input: current `crates/kamn-core/src/runtime_peer_coordination.rs` production source plus existing runtime and integration tests that import its public surface through `runtime.rs` and `lib.rs`
- Output: a thin root shell that delegates to bounded sibling modules for the extracted peer-coordination domains
- Output: a hard-fail extraction contract enforcing the new module layout

## Boundaries/Non-goals
- Do not change runtime coordination semantics or public API names
- Do not redesign adjacent runtime modules outside the extraction seams required by this file
- Do not add new peer-coordination features, new config surface, or new dependencies
- Do not weaken or delete existing runtime/integration tests to make the split pass

## Failure modes
- Extraction contract passes while `runtime_peer_coordination.rs` remains oversized or expected modules are missing
- Public re-export surface drifts and breaks `runtime.rs`, `lib.rs`, or downstream tests silently
- Lifecycle, queue/backpressure, authenticated peer-frame, proposal planner, or runtime wiring behavior changes during extraction
- Any touched extracted file exceeds the touched-Rust size policy
- Final branch still fails touched-Rust size policy

## Acceptance criteria
- [x] `crates/kamn-core/src/runtime_peer_coordination.rs` becomes a thin root shell under the active file-size policy
- [x] bounded sibling modules exist for lifecycle/queue, peer-frame auth/signing, proposal planning, runtime wiring/transport profile, and tests
- [x] the runtime and lib re-export surface remains wired and downstream compilation continues to succeed
- [x] a hard-fail extraction contract exists and passes
- [x] real runtime and integration coverage for the touched domains still passes after the split
- [x] touched-Rust size policy returns `GO` on the final branch

## Files to touch
- `crates/kamn-core/src/runtime_peer_coordination.rs`
- `crates/kamn-core/src/runtime_peer_coordination/`
- `crates/kamn-core/tests/runtime_peer_coordination_module_extraction_contract.rs`
- `specs/6901-split-runtime-peer-coordination.md`

## Error semantics
- No new fallbacks or swallowed failures in lifecycle, queue, peer-frame, planner, or wiring paths
- Existing typed error behavior remains fail-closed and externally observable through current return types
- Environment/config validation for peer-frame signing remains startup/runtime-fail-loud behavior

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing
- Run the extraction contract target
- Run the runtime peer lifecycle/integration targets that cover the touched domains after the split
- Run touched-Rust size policy on the final branch

## Final evidence
- `cargo test -p kamn-core --test runtime_peer_coordination_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test authenticated_peer_frame_integration -- --nocapture`
- `cargo test -p kamn-core --test runtime_wiring_transport_profile_integration -- --nocapture`
- `cargo test -p kamn-core --test peer_lifecycle_proptest_invariants -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6901-touched-size-final.json`

## Deviations
- Clean-clone verification was not required for this issue because the working checkout remained isolated to the issue write set and the direct Python touched-Rust entrypoint returned `policy_decision=GO`.
