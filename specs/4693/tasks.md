# Issue #4693 Tasks

- Issue: `#4693`
- Status: `InProgress`

## Ordered Tasks
- T1 (Red, Functional/Regression): add/retain failing assertions for transport/pipeline ownership drift and reason-code drift.
- T2 (Green, Refactor): extract next `p2p_transport` responsibility slice into dedicated module(s) with stable exports.
- T3 (Green, Refactor): extract next `block_pipeline` responsibility slice into dedicated module(s) with stable exports.
- T4 (Regression): run scoped kamn-core transport/pipeline tests and resolve drift.
- T5 (Docs): update runtime-network ownership mapping for transport/pipeline boundaries.
- T6 (Verify): run
  - `cargo fmt --check`
  - `cargo test -p kamn-core --test p2p_transport_runtime -- --nocapture`
  - `cargo test -p kamn-core --test block_pipeline -- --nocapture`

## Current Slice Status
- T1: ✅ completed via `cargo test -p kamn-core --test p2p_block_module_extraction_contract -- --nocapture` red failure before extraction.
- T2: ✅ completed for transport ownership extraction into:
  - `crates/kamn-core/src/p2p_transport/adapter.rs`
  - `crates/kamn-core/src/p2p_transport/coordinator.rs`
  - `crates/kamn-core/src/p2p_transport/runtime_event.rs`
  - `crates/kamn-core/src/p2p_transport/swarm_stack.rs`
  - `crates/kamn-core/src/p2p_transport/native_runtime.rs`
- T3: ✅ completed for fork-choice and canonical commit-store ownership extraction into:
  - `crates/kamn-core/src/block_pipeline/fork_choice.rs`
  - `crates/kamn-core/src/block_pipeline/evidence.rs`
  - `crates/kamn-core/src/block_pipeline/commit_store.rs`
- T4: ✅ completed for current slices with scoped transport/pipeline/fork-choice/commit-store regression suites.
- T5: ✅ completed for current slices with ownership map updates in `docs/foundation/runtime-network.md`.
- T6: ✅ completed for current slices (`cargo fmt --check`, scoped tests, and `cargo clippy -p kamn-core -- -D warnings`).

## Completion Evidence
- Core monolith files are reduced with explicit extracted module ownership.
- Scoped transport/pipeline suites pass with deterministic fail-closed behavior preserved.
