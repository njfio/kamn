# 6843-split-p2p-transport-live

## Objective
Reduce `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs` from a 1711 LOC production monolith to a thin root shell plus bounded concern modules without changing libp2p live transport behavior, deterministic config behavior, bootstrap planning, regression harness behavior, or exported API surface.

## Inputs/Outputs
- Input: existing live-runtime inbox/backpressure helpers, live peer lifecycle transport implementation, native adapter/swarm loop logic, deterministic config/bootstrap helpers, regression harness/report code, and inline tests in `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- Output: bounded sibling modules and a root shell that preserves the public `kamn_core::p2p_transport::p2p_transport_live` surface
- Output: structural ratchet coverage that enforces the staged extraction layout on touched code

## Boundaries/Non-goals
- No new dependencies
- No protocol or wire-format behavior changes
- No changes to libp2p backend resolution semantics
- No changes to regression corpus meaning or reporting fields beyond module movement required for extraction
- No weakening of existing tests or invariants

## Failure modes
- Root shell remains above the staged extraction cap
- Any touched extracted file exceeds 200 LOC
- Any touched function exceeds 25 LOC
- Public exports drift and downstream crates fail to compile
- Live transport runtime, swarm loop, deterministic config, or regression harness behavior changes during extraction
- Touched-Rust size policy remains `NO-GO`

## Acceptance criteria
- [x] `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs` is reduced to a thin root shell at or below a staged extraction cap defined by the red contract
- [x] Extracted sibling modules are organized by concern rather than arbitrary line slicing
- [x] All touched extracted files remain at or below 200 LOC
- [x] All touched functions remain at or below 25 LOC
- [x] Existing libp2p live transport behavior, deterministic config behavior, bootstrap planning, and regression harness behavior remain unchanged
- [x] Existing tests that exercise the live transport surface still pass
- [x] At least one new extraction contract enforces the staged root shell and module layout
- [x] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root <repo-root> --base-ref origin/main --output-json <path>` returns `policy_decision=GO`

## Files to touch
- `specs/6843-split-p2p-transport-live.md`
- `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- `crates/kamn-core/src/p2p_transport/p2p_transport_live/`
- `crates/kamn-core/tests/p2p_transport_live_module_extraction_contract.rs`

## Error semantics
- Preserve all existing `P2pTransportError`, regression error, and runtime failure behavior unless a failing test proves the old behavior was already wrong
- Preserve fail-closed behavior for invalid transport configuration, invalid topic/network identifiers, backpressure failures, and swarm/runtime loop failures
- Structural contract failures must fail with explicit missing-file, missing-marker, or root-budget assertions

## Test plan
1. Add a red extraction contract for `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`.
2. Split the file into bounded modules by concern:
   - runtime inbox/backpressure helpers
   - live peer lifecycle transport implementation
   - native adapter/data-plane/swarm loop wiring
   - deterministic config and behavior/bootstrap composition helpers
   - regression corpus, case runner, and harness/report code
3. Run the extraction contract and the existing `p2p_transport_live` test lane.
4. Run the touched-Rust size ratchet and require `policy_decision=GO`.

## Phase 6 Evidence
- Root shell reduced to `37` LOC:
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- Extracted sibling modules:
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live/deterministic_config.rs`
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live/native_runtime_loop.rs`
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live/peer_lifecycle_transport.rs`
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live/peer_lifecycle_transport/contract_data_plane.rs`
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live/regression_harness.rs`
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live/runtime_inbox.rs`
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live/swarm_runtime.rs`
- Verified commands:
  - `cargo test -p kamn-core --test p2p_transport_live_module_extraction_contract -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture`
  - `TMPDIR=/home/n/Code/kamn/tmp CARGO_TARGET_DIR=/home/n/Code/kamn/target cargo test -p kamn-core --features libp2p-live-transport --test p2p_libp2p_native_adapter_runtime -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /tmp/kamn-6840-remote --base-ref origin/main --output-json /tmp/6843-touched-size-post-refactor.json`
- Final touched-Rust result:
  - `policy_decision=GO`

## Deviations
- No behavior deviations from the spec were required.
- The feature-path visibility fixes touched `crates/kamn-core/src/p2p_transport/native_runtime.rs` and `crates/kamn-core/src/p2p_transport.rs` in addition to the initial file list so the extracted live-transport modules could reuse the already-extracted sibling runtime/swarm code without duplication.
