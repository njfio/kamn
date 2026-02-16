# Issue #4693 Plan

- Issue: `#4693`
- Status: `InProgress`

## Approach
- Add extraction-boundary regression tests (RED) for `p2p_transport` and `block_pipeline` root module declarations.
- Extract `p2p_transport` live/libp2p runtime internals into dedicated submodule(s), preserving root API through re-exports.
- Extract `block_pipeline` ingress/store/fork-choice support internals into dedicated submodule(s), preserving root API through re-exports.
- Update runtime-network docs with explicit ownership mapping and verification commands.

## Affected Modules
- `crates/kamn-core/src/p2p_transport.rs`
- `crates/kamn-core/src/p2p_transport/*.rs` (new)
- `crates/kamn-core/src/block_pipeline.rs`
- `crates/kamn-core/src/block_pipeline/*.rs` (new)
- `crates/kamn-core/tests/transport_pipeline_module_extraction_contract.rs` (new)
- `docs/foundation/runtime-network.md`

## Risks and Mitigations
- Risk: extraction introduces visibility/import drift.
- Mitigation: preserve API via explicit re-exports and run targeted selectors before full gate checks.
- Risk: deterministic reason-code behavior regresses.
- Mitigation: run existing reason-code regression selectors and keep error surface unchanged.
- Risk: feature-gated libp2p code path compile drift.
- Mitigation: keep feature-gated code grouped in extracted submodule and run clippy/tests on `kamn-core` crate.

## Interface Contract
- No external API or wire contract changes.
- Public exports from `kamn-core::p2p_transport` and `kamn-core::block_pipeline` remain stable.

## ADR
- Not required (no dependency/protocol change).
