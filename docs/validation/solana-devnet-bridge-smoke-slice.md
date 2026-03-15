# Solana Devnet Bridge Smoke Slice

This runbook documents one bounded Solana devnet-addressed bridge smoke slice on current `main`. It proves an executable Solana bridge path on the public bridge surface using the devnet route id `solana:devnet:program:task-v1`. It is not live Solana RPC/devnet-backed proof and it does not claim external settlement.

## What This Slice Proves
- the public bridge surface accepts and routes the Solana devnet-addressed channel id `solana:devnet:program:task-v1`
- `solana_quorum_dispatches_outbound` proves a Solana outbound quorum dispatch on the public cross-chain bridge engine
- `integration_projects_solana_inbound_to_envelope` proves Solana inbound projection to a canonical message envelope
- `integration_outbound_quorum_rejections_are_explicit_and_fail_closed` proves fail-closed under-quorum and unauthorized-approver rejection on the Solana outbound path
- `regression_rejects_replayed_solana_inbound_projection_event` proves fail-closed replay rejection for the Solana inbound path

## What This Slice Does Not Prove
- not live Solana RPC/devnet-backed proof
- not external chain settlement
- not bridge finality beyond the bounded receipt-normalization proof already documented elsewhere
- not multi-node or production-readiness claims

## Executable Anchors
- `crates/kamn-core/tests/cross_chain_bridge.rs`
  - `solana_quorum_dispatches_outbound`
  - `integration_projects_solana_inbound_to_envelope`
  - `regression_rejects_replayed_solana_inbound_projection_event`
- `crates/kamn-core/tests/bridge_outbound_quorum_execution.rs`
  - `integration_outbound_quorum_rejections_are_explicit_and_fail_closed`

## How To Run
```bash
cargo test -p kamn-core --test cross_chain_bridge -- --nocapture
cargo test -p kamn-core --test bridge_outbound_quorum_execution -- --nocapture
cargo test -p kamn-node --test solana_devnet_bridge_smoke_slice_contract -- --nocapture
```

## Observed Behavior
- the Solana devnet route id is configured as `solana:devnet:program:task-v1`
- outbound quorum success produces a `BridgePlatform::Custom("solana")` dispatch with two required approvals
- inbound Solana projection produces a canonical message envelope for the configured target DID
- replay rejection remains explicit as `duplicate inbound message id: solana:slot-881991:ix-2`
- under-quorum, unauthorized-approver, and replay cases fail closed rather than degrading silently

## Bound
This slice proves a bounded Solana devnet-addressed bridge smoke path on the public bridge surface. A future issue must prove any real live Solana devnet-backed path separately.
