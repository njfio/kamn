# Cross-Chain Bridge Adapters (Ethereum and Solana) (Issues #232, #233, #740, #742)

This document captures cross-chain adapter and receipt-finality normalization slices for Ethereum, Solana, and Near pathways.

## Scope Delivered
- Added `crates/kamn-core/src/cross_chain_bridge.rs` with:
  - `CrossChainNetwork` enum (`Ethereum`, `Solana`).
  - `CrossChainBridgeConfig` for listener and approver allowlists, approval threshold, and chain route maps.
  - `CrossChainBridgeEngine` that wires chain-specific adapter engines.
  - `process_inbound(...)` and `process_inbound_to_envelope(...)` with listener and route validation.
  - `process_outbound_with_approvals(...)` with approver quorum enforcement and deterministic approval metadata.
  - typed error handling via `CrossChainBridgeError`.
- Added integration tests in `crates/kamn-core/tests/cross_chain_bridge.rs`.
- Added `crates/kamn-core/src/cross_chain_receipt.rs` with deterministic Ethereum/Near receipt-finality normalization:
  - `CrossChainReceiptNetwork` enum (`Ethereum`, `Near`).
  - `CrossChainReceiptProof` and `NormalizedCrossChainReceipt` contract structures.
  - `normalize_cross_chain_receipt(...)` with strict chain-specific finality rules.
  - typed errors via `CrossChainReceiptNormalizationError`.
- Added integration tests in `crates/kamn-core/tests/cross_chain_receipt_finality.rs`.
- Added outbound cross-chain intent evidence scripts:
  - `scripts/bridge/generate_cross_chain_outbound_intent_evidence_bundle.sh`
  - `scripts/bridge/check_cross_chain_outbound_intent_policy.sh`
  - `scripts/bridge/run_cross_chain_outbound_intent_contract_lane.sh`
  - `scripts/bridge/run_cross_chain_outbound_intent_deep_lane.sh`
  - `scripts/bridge/run_cross_chain_outbound_intent_matrix.py`
  - fixture matrix: `fixtures/bridge_outbound_intent/approval_retry_cases.json`

## Validation Rules
- Bridge/listener/approver DIDs must parse as `kamn:did:agent:*`.
- Listener and approver allowlists must be non-empty.
- `required_approvals` must be between `1` and approver allowlist size.
- Ethereum and Solana route maps must both be present and non-empty.
- Inbound processing requires:
  - authorized listener DID.
  - route lookup hit for chain-specific `external_channel_id`.
  - `target_agent_did` equal to mapped route target DID.
- Outbound processing requires:
  - destination channel present in chain-specific route map.
  - authorized approver set with no duplicates.
  - provided approvals satisfy quorum threshold.

## Receipt Finality Normalization Rules (Ethereum / Near)
- Shared input validation:
  - `receipt_id`, `block_reference`, and `finality_label` must be non-empty.
- Ethereum success normalization:
  - `finalized` or `safe` with at least `12` confirmations -> `Final`.
  - `finalized`, `safe`, or `latest` below threshold -> `Pending`.
  - unknown labels are rejected.
- Near success normalization:
  - `final` -> `Final`.
  - `optimistic` or `none` -> `Pending`.
  - unknown labels are rejected.
- Status override behavior:
  - `Failed` status always normalizes to `Failed`.
  - `Pending` status always normalizes to `Pending`.

## Outbound Intent Attestation and Retry Idempotency Rules
- Outbound intent evidence requires:
  - chain (`ethereum` or `near`) and destination channel prefix alignment.
  - quorum evidence (`required` approvals met by `received` approvals).
  - non-empty `approval_quorum_hash` (`sha256:...`).
- Retry/idempotency constraints:
  - idempotency key must be `idemp:<value>`.
  - attempt number must be at least `1`.
  - retry attempts (`attempt_number > 1`) must preserve payload hash equality.
  - duplicate request flag forces deterministic `NO-GO`.
- Policy checker behavior:
  - recomputes expected final decision from bundle fields.
  - rejects tampered `final_decision` mismatches fail closed (`Regression: #742`).

## Processing Flow
- `process_inbound(...)`:
  - validates listener and chain route mapping.
  - normalizes payload through chain-specific bridge adapter engine.
- `process_inbound_to_envelope(...)`:
  - reuses inbound validation path.
  - projects inbound message to canonical KAMN envelope.
- `process_outbound_with_approvals(...)`:
  - validates route + approver quorum.
  - translates outbound payload via selected chain adapter engine.
  - returns translated payload and approval metadata for audit trails.

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test cross_chain_bridge
cargo test -p kamn-core --test cross_chain_receipt_finality
bash scripts/bridge/test_generate_cross_chain_outbound_intent_evidence_bundle.sh
bash scripts/bridge/test_run_cross_chain_outbound_intent_contract_lane.sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```
