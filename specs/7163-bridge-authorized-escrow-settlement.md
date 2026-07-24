# 7163 Bridge-Authorized Escrow Settlement

## Objective

Authorize terminal escrow settlement only from one finalized bridge receipt
that is bound to the same escrow, actor, recipient, amount, network, and task
terms, while preserving exactly one economic transfer.

## Inputs / Outputs

Inputs:
- finalized canonical bridge receipt from #7160
- escrow ID, task ID, releasing actor, recipient, amount, asset, network, and
  terms digest
- durable settlement intent and receipt-chain state

Outputs:
- settlement intent bound to the bridge receipt ID, digest, and signature
- terminal escrow state backed by the existing bridge transfer
- task projection and independent-verifier evidence for the same binding

## Boundaries / Non-Goals

- Depends on #7160 and must not synthesize a bridge receipt.
- Do not submit a second transfer during escrow release.
- Do not implement adapter parity; that is #7161.
- Solana devnet is the first bounded live proof.
- Do not claim mainnet custody, generalized settlement, bridge consensus, or
  production readiness.
- Do not add a dependency or weaken existing direct-settlement rejection tests.

## Failure Modes

- Bridge receipt is missing, pending, failed, malformed, or not authoritative.
- Receipt escrow, task, actor, recipient, amount, asset, network, or terms
  digest does not match the release request.
- Receipt belongs to another resource or has already authorized a different
  settlement.
- Receipt order or chain membership is invalid.
- Release retry or restart attempts another transfer.
- Terminal escrow state is written before receipt validation completes.
- Task projection or verifier omits or disagrees with the bridge binding.

## Acceptance Criteria

- [ ] Release requires a finalized service bridge receipt bound to the same
      escrow and complete economic terms.
- [ ] The settlement intent digest includes bridge ID, bridge receipt ID and
      digest, transaction signature, escrow ID, task ID, actor, recipient,
      amount, asset, network, and terms digest.
- [ ] Release consumes the bridge transfer and submits no second transaction.
- [ ] Missing, pending, failed, replayed, cross-resource, cross-actor, reordered,
      and tampered receipts hard-fail before terminal persistence.
- [ ] Retry, timeout reconciliation, and restart return the same settlement
      receipt and transaction signature with one transfer.
- [ ] Task projection and the standalone verifier expose and recompute the same
      bridge-to-settlement receipt chain.
- [ ] Deterministic and ignored-live integration tests prove the real path.

## Files To Touch

Expected production surface:
- escrow release live-settlement routing
- settlement intent and receipt persistence in the task/escrow store
- authority digest construction and task projection receipt-chain code
- standalone verifier bridge-to-settlement validation

Proof surface:
- `docs/validation/bridge-authorized-escrow-settlement-slice.md`
- focused unit, service-route, restart, verifier, and ignored-live tests

Exact paths are selected during RED after mapping current module ownership.

## Error Semantics

- `BRIDGE_SETTLEMENT_AUTHORITY_MISSING`: no finalized bridge receipt is present.
- `BRIDGE_SETTLEMENT_AUTHORITY_MISMATCH`: receipt authority exists but resource,
  actor, or economic terms do not match.
- `BRIDGE_SETTLEMENT_RECEIPT_REPLAY`: the receipt is already consumed by another
  settlement intent.
- Existing settlement-evidence errors remain for correctly authorized receipts
  whose settlement facts are internally invalid.

Authority validation precedes settlement-evidence validation. Interior code
returns typed errors; the service boundary logs once and translates the error.
Failures never write terminal escrow state.

## Test Plan

Red:
- Full authority mismatch matrix, receipt order/membership, replay, and
  one-transfer idempotence tests.
- Projection and standalone-verifier rejection tests.
- Documentation contract before the bounded proof exists.

Green:
- Extend the settlement intent with the minimum bridge-authority bindings.
- Consume the existing finalized transfer and persist one terminal receipt.

Refactor:
- Centralize bridge/settlement binding fields and digest construction.
- Separate authority validation from settlement fact validation.
- Verify file/function size, naming, duplication, and idempotency.

Integration:
- Exercise bridge finality through escrow release, task projection, restart,
  and independent verification.
- Run one ignored live devnet proof and independently confirm one transfer.
