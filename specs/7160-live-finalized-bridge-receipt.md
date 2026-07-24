# 7160 Live Finalized Bridge Receipt

## Objective

Persist one service-authoritative bridge receipt whose terminal finality is
derived from a real finalized Solana devnet transaction bound to the original
bridge intent and source message.

## Inputs / Outputs

Inputs:
- authenticated bridge-submit and bridge-forward requests
- bridge ID, source message ID, source network, target network, and payload hash
- validated live Solana RPC endpoint and funded devnet signer
- submitted transaction bytes/signature and authoritative RPC response

Outputs:
- a durable bridge receipt containing receipt ID/digest, bridge and message
  bindings, transaction signature, network, commitment, finalized slot, action,
  resource ID, and resulting state
- restart-visible terminal bridge state derived from that receipt
- one secret-safe opt-in live proof artifact

## Boundaries / Non-Goals

- Solana devnet only.
- Do not claim mainnet, custody, generalized cross-chain finality, consensus, or
  production readiness.
- Do not gate escrow release in this issue; that is #7163.
- Do not change SDK/CLI/MCP parity in this issue; that is #7161.
- Do not add a dependency.
- Do not treat a finalized slot observation or deterministic local hash as a
  submitted transaction receipt.

## Failure Modes

- Transaction preparation is not persisted before submission.
- RPC submission returns an ambiguous response.
- RPC evidence is pending, failed, absent, malformed, or from another network.
- Signature, bridge ID, source message, target network, or payload hash differs
  from the prepared intent.
- A finalized receipt is replayed for another bridge or message.
- Retry or restart submits a second transaction instead of reconciling.
- Terminal finality is persisted before authoritative evidence validates.

## Acceptance Criteria

- [x] Live mode prepares and durably records one transaction bound to bridge ID,
      source message ID, target network, and payload hash before submission.
- [x] Pending, failed, unknown, malformed, or mismatched evidence cannot produce
      terminal finalized bridge state.
- [x] Valid finalized evidence produces a canonical service bridge receipt with
      a recomputable digest and authoritative transaction fields.
- [x] Retry, restart, timeout, and ambiguous-response recovery reconcile the
      prepared transaction before any resubmission.
- [x] A repeated operation returns the same receipt and signature and observes
      exactly one submitted transfer.
- [x] An ignored live devnet test independently queries RPC and verifies the
      persisted receipt.
- [x] A bounded runbook and documentation contract preserve exact non-claims.

## Files To Touch

Expected production surface:
- `crates/kamn-node/src/service_api_endpoint/live_bridge_dispatch.rs`
- bridge service models, payload parsing, and message-store bridge operations
- focused bridge route/restart tests under `crates/kamn-node/tests/`

Proof surface:
- `docs/validation/live-chain-backed-bridge-finality-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- focused deterministic and ignored-live integration contracts

Exact file splits must preserve the 200-line file and 25-line function limits.

## Error Semantics

- `BRIDGE_FINALITY_PENDING`: authoritative evidence is valid but non-terminal.
- `BRIDGE_FINALITY_EVIDENCE_INVALID`: evidence is malformed, failed, or does
  not bind the prepared bridge intent.
- `BRIDGE_RECEIPT_REPLAY`: a receipt is already bound to another resource.
- `BRIDGE_RECONCILIATION_REQUIRED`: submission outcome is ambiguous and must be
  reconciled before retry.

Interior code returns typed errors without logging. The service boundary logs
once with correlation context and returns a structured hard failure. No
terminal state is written after an error.

## Test Plan

Red:
- Binding matrix for bridge/message/network/payload/signature mismatches.
- Pending, failed, malformed, replay, ambiguous-response, retry, and restart
  tests.
- A documentation contract that fails before the live proof exists.

Green:
- Reuse prepared-transaction and reconcile-before-resubmit patterns from live
  settlement.
- Persist the minimum canonical receipt fields and terminal transition.

Refactor:
- Separate transaction preparation, RPC transport, evidence validation,
  receipt construction, and persistence.
- Verify size, naming, duplication, error, and idempotency checks.

Integration:
- Exercise the real service bridge entrypoint and persistence layer.
- Run the ignored live devnet submission/finality/restart proof.
- Independently query RPC for the exact persisted signature.

## Verification Evidence

- `cargo test -p kamn-node --test live_chain_backed_bridge_finality_contract`
- `cargo test -p kamn-node --bin kamn-node live_bridge_contract_tests`
- `cargo test -p kamn-node --bin kamn-node transport_tests`
- ignored live proof command in
  `docs/validation/live-chain-backed-bridge-finality-slice.md`
- secret-safe result in
  `docs/validation/evidence/7160-live-bridge-finality.json`
