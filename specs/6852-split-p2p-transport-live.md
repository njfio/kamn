# Objective
Split `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs` into bounded, concern-based modules while preserving live transport behavior, feature-flag semantics, runtime event behavior, and existing native adapter/runtime tests.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
  - existing adjacent transport modules in `crates/kamn-core/src/p2p_transport/`
  - current live transport and native adapter tests
- Outputs:
  - a thin `p2p_transport_live.rs` root shell under the active file-size budget
  - bounded sibling modules for runtime inbox/backpressure, peer lifecycle operations, deterministic config/runtime resolution, and regression/test support
  - extraction contract coverage enforcing the new module layout and active size limits

## Boundaries/Non-goals
- Do not change transport behavior or feature-flag semantics.
- Do not duplicate logic already extracted into adjacent transport modules.
- Do not introduce new dependencies.
- Do not change public error semantics.

## Failure modes
- Root file remains oversized after the split.
- Extracted modules duplicate or drift from `native_runtime`, `swarm_stack`, or `lifecycle_regression` behavior.
- Runtime event emission or live inbox backpressure behavior changes.
- Extracted files or functions still exceed active size limits.

## Acceptance criteria
- [x] The root file is reduced to a thin shell under the active file-size budget.
- [x] Runtime inbox / peer lifecycle / native runtime resolution / deterministic config / regression seams are extracted into bounded modules.
- [x] Existing live transport and native adapter tests remain green.
- [x] No extracted file exceeds the active file-size limit.
- [x] No extracted function exceeds the active function-size limit.

## Files to touch
- `specs/6852-split-p2p-transport-live.md`
- `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- `crates/kamn-core/tests/p2p_transport_live_module_extraction_contract.rs`
- optional sibling modules under `crates/kamn-core/src/p2p_transport/p2p_transport_live/`

## Error semantics
- Existing `P2pTransportError` behavior remains fail-closed.
- Live inbox backpressure rejection and purge semantics must remain deterministic.
- No silent fallback behavior may be introduced during the split.

## Test plan
1. Add a red extraction contract that fails while the root file remains oversized and the planned module layout is absent.
2. Re-run the existing live transport/native adapter targets to preserve behavior while splitting.
3. Extract the file into bounded concern-based modules.
4. Re-run the extraction contract and live transport targets until green.
5. Run the clean-clone touched-Rust size ratchet on the final write set.

## Final evidence
- Root shell reduced to `35` LOC at `crates/kamn-core/src/p2p_transport/p2p_transport_live.rs`
- Extracted modules:
  - `runtime_inbox.rs`
  - `peer_lifecycle_transport.rs`
  - `peer_lifecycle_transport/contract_data_plane.rs`
  - `native_runtime_loop.rs`
  - `swarm_runtime.rs`
  - `deterministic_config.rs`
  - `regression_harness.rs`
- Verification:
  - `cargo test -p kamn-core --test p2p_transport_live_module_extraction_contract -- --nocapture`
  - `cargo test -p kamn-core --test p2p_live_transport_runtime -- --nocapture`
  - `cargo test -p kamn-core --features libp2p-live-transport --test p2p_libp2p_native_adapter_runtime -- --nocapture`
  - `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-6850-remote-clean4 --base-ref origin/main --output-json /tmp/6852-touched-size-refactor.json`
- Touched-Rust result: `policy_decision=GO`

## Deviations
- Clean-clone validation used the Python touched-Rust entrypoint directly instead of the shell wrapper so the repo root stayed pinned to `/home/n/Code/kamn-6850-remote-clean4`.
