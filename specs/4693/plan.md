# Issue #4693 Plan

- Issue: `#4693`
- Status: `InProgress`

## Approach
- Continue staged extraction pattern already used in runtime/signer decomposition:
  - move cohesive helpers into `p2p_transport/` and `block_pipeline/` modules,
  - keep outer orchestrator functions in root file,
  - preserve callsites and reason-taxonomy strings.
- Keep each extraction slice test-backed and small enough for deterministic review.

## Affected Modules
- `crates/kamn-core/src/p2p_transport.rs`
- `crates/kamn-core/src/p2p_transport/adapter.rs`
- `crates/kamn-core/src/p2p_transport/validation.rs`
- `crates/kamn-core/src/p2p_transport/lifecycle_regression.rs`
- `crates/kamn-core/src/p2p_transport/error.rs`
- `crates/kamn-core/src/p2p_transport/runtime_event.rs`
- `crates/kamn-core/src/p2p_transport/swarm_stack.rs`
- `crates/kamn-core/src/p2p_transport/native_runtime.rs`
- `crates/kamn-core/src/block_pipeline.rs`
- `crates/kamn-core/src/block_pipeline/validation.rs`
- `crates/kamn-core/src/block_pipeline/gossip_ingress.rs`
- `crates/kamn-core/src/block_pipeline/fork_choice.rs`
- `crates/kamn-core/src/block_pipeline/evidence.rs`
- `crates/kamn-core/src/block_pipeline/commit_store.rs`
- `crates/kamn-core/tests/p2p_block_module_extraction_contract.rs`
- `docs/foundation/runtime-network.md`

## Risks and Mitigations
- Risk: extraction breaks implicit transport/pipeline invariants.
- Mitigation: keep strict parity assertions in scoped runtime tests.
- Risk: reason-code drift in fail-closed paths.
- Mitigation: avoid string changes and run regression-focused tests.
- Risk: partial extraction leaves ownership ambiguous.
- Mitigation: document explicit module ownership in docs and issue evidence.

## Interface Contract
- Public runtime APIs remain stable; extraction is internal module ownership refactoring.
- Existing transport and pipeline tests continue to gate behavior parity.

## ADR
- No ADR required: no dependency, protocol, or public API contract change.
