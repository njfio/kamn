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
- [ ] `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs` is reduced to a thin root shell at or below a staged extraction cap defined by the red contract
- [ ] Extracted sibling modules are organized by concern rather than arbitrary line slicing
- [ ] All touched extracted files remain at or below 200 LOC
- [ ] All touched functions remain at or below 25 LOC
- [ ] Existing libp2p live transport behavior, deterministic config behavior, bootstrap planning, and regression harness behavior remain unchanged
- [ ] Existing tests that exercise the live transport surface still pass
- [ ] At least one new extraction contract enforces the staged root shell and module layout
- [ ] `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root <repo-root> --base-ref origin/main --output-json <path>` returns `policy_decision=GO`

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
