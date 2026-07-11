# Make Devnet Settlement Idempotent And Recoverable

## Objective

Bind exactly one Solana devnet transfer to one task-bound escrow release. Persist
the transfer identity before submission, reconcile known signatures after
timeouts or restart, and never report `released` without confirmed evidence
matching the escrow agreement.

## Inputs/Outputs

The signed release request supplies an `idempotency_key`. The authenticated
release authority, escrow ID, beneficiary, amount, and network come from the
verified #7091 escrow agreement. Live configuration supplies the external
keypair path, RPC URL, recipient public key, lamports, and commitment.

A durable settlement intent contains:

- intent ID, escrow ID, release idempotency key, and actor DID
- recipient public key, amount, and `solana:devnet` network
- deterministic signed transaction signature and signed transaction digest
- `prepared`, `submitted`, `confirmed`, `reconciled`, `ambiguous`, or `failed`
  state
- submission attempt count and optional structured error code
- confirmed commitment and balance/recipient evidence required by the existing
  live proof surface

Success returns the escrow in `released` state with the confirmed signature,
network, commitment, and receipt linkage. Ambiguous or failed outcomes return a
structured error and never return `released`.

## Boundaries/Non-goals

- Solana devnet only; no mainnet or economic-value claim.
- No production custody, key management, dispute, refund, or multi-chain work.
- No local simulation, dry-run, placeholder signature, or slot-only evidence
  can satisfy asset-movement success.
- Reuse the existing live settlement adapter, service API store, and proof
  surfaces. Do not add a generalized workflow engine or blockchain adapter.
- External keypair paths remain runtime-only and never enter git or artifacts.
- An expired or never-landed signed transaction remains recoverable only by
  querying or resubmitting that exact transaction. This MVP does not generate a
  replacement signature because doing so would weaken at-most-once settlement.

## Settlement Contract

Before submission KAMN builds and signs the exact transfer transaction, derives
its stable signature and digest, and persists a `prepared` intent. The intent
must match the escrow beneficiary, amount, network, and release key. Conflicting
reuse returns `SETTLEMENT_INTENT_CONFLICT` without submission.

On first execution KAMN submits the already-signed transaction and increments
the durable attempt count. Solana signatures identify the transaction, so retry
must query the known signature before considering any submission. Resubmission
may only send the identical signed transaction; KAMN must never build another
transfer for an existing intent.

Confirmed evidence must prove:

- cluster/network is Solana devnet
- signature equals the persisted expected signature
- transaction recipient and lamports equal the escrow agreement
- configured commitment is satisfied
- existing balance evidence is internally consistent

Only then may the intent become `confirmed`/`reconciled` and escrow become
`released` in one durable state write.

## Crash And Retry Semantics

- Failure before durable `prepared` state: no submission and hard failure.
- RPC failure known to occur before submission: intent becomes `failed`; no
  success is reported.
- Timeout or persistence failure after possible submission: intent becomes
  `ambiguous`, returns `SETTLEMENT_OUTCOME_AMBIGUOUS`, and never reports release.
- Restart with `prepared`, `submitted`, or `ambiguous`: query the persisted
  signature first and converge to confirmed, failed, or still ambiguous.
- A transaction that remains absent after its blockhash expires stays
  observably `ambiguous`; operator-driven replacement is roadmap work and is
  not claimed by this issue.
- Concurrent identical requests serialize through the durable intent and return
  one result; they cannot create a second transaction identity.
- Proof/report regeneration reads persisted evidence and never submits.

## Failure Modes

- Conflicting key or agreement: `SETTLEMENT_INTENT_CONFLICT` (409).
- Missing or noncanonical escrow agreement: existing escrow lifecycle error.
- Config recipient/amount/network mismatch: `SETTLEMENT_AGREEMENT_MISMATCH` (409).
- Signature, recipient, amount, or network evidence mismatch:
  `SETTLEMENT_EVIDENCE_MISMATCH` (500).
- Unknown post-submit outcome: `SETTLEMENT_OUTCOME_AMBIGUOUS` (503).
- Confirmed transaction failure: `SETTLEMENT_TRANSACTION_FAILED` (502).
- Persistence error before submission: existing persistence error (500), with
  no adapter call.

All errors are observable and hard-fail. No fallback can mark an escrow
released.

## Acceptance Criteria

- [x] Intent is durably `prepared` before the external submit boundary.
- [x] Intent binds escrow, actor, recipient, bounded amount, devnet, release key,
      expected signature, and signed transaction digest.
- [x] Identical retry reconciles the known signature before any resubmission.
- [x] Conflicting reuse returns `SETTLEMENT_INTENT_CONFLICT` without mutation.
- [x] Confirmed evidence validates signature, recipient, amount, devnet, and
      commitment before release.
- [x] Ambiguous post-submit outcomes never report `released`.
- [x] Restart reconciliation converges without creating another transaction.
- [x] Concurrent release requests share one intent and transaction identity.
- [x] Proof generation cannot invoke submission.
- [x] Negative authorization and retry tests pass before funded rehearsal.
- [x] One bounded funded devnet rehearsal proves exactly one confirmed transfer.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint/message_store/` focused settlement
  intent models and persistence operations
- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch/`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/update_routes/state_routes_release.rs`
- task/escrow settlement, restart, concurrency, and live devnet contracts
- MVP devnet settlement report/verifier only where persisted intent evidence is
  required for an existing claim

## Error Semantics

Interior settlement code returns typed errors containing stable code, message,
context IDs, and source cause where applicable. The HTTP entrypoint translates
once. Intent persistence happens before network calls. Post-submit persistence
failure must retain enough deterministic identity for reconciliation and must
not be translated into ordinary RPC failure or local authorization success.

## Test Plan

RED:

- Test adapter observes no persisted intent when submission begins.
- A post-submit persistence failure followed by retry submits again.
- Concurrent release requests can produce multiple adapter calls.
- Restart cannot reconcile prepared/submitted/ambiguous state by signature.
- Mismatched signature/recipient/amount evidence can mark escrow released.

GREEN:

- Add the minimal serde-defaulted settlement intent record and typed errors.
- Split transaction preparation from submit/confirm so identity is known first.
- Persist intent, submit the exact transaction, reconcile by signature, then
  atomically persist confirmed intent and released escrow.

REFACTOR:

- Separate preparation, persistence, submission, reconciliation, and evidence
  validation into focused modules under 200 lines and functions under 25 lines.
- Keep the existing adapter boundary and dependency set.

INTEGRATION:

- Signed HTTP tests cover conflicts, no-submit denials, crash windows, restart,
  concurrency, and proof retry.
- Run targeted contracts, full `kamn-node`, formatting, strict workspace clippy,
  and `make check`.
- Run one real bounded transfer with the external funded devnet keypair. Record
  signature, recipient, lamports, commitment, balances, persisted intent, and
  verifier output. If RPC/keypair/funding is unavailable, record explicit
  `NO-GO` and do not claim issue completion.

## Rollback

Each phase is a separate commit. Before live submission, retain a checkpoint
with all deterministic tests green. A live rehearsal uses a fresh release key
and bounded amount. Never delete state containing `submitted` or `ambiguous`;
reconcile it by the persisted signature before rollback or another rehearsal.

## Completion Evidence

The final escrow-bound required-mode rehearsal is preserved at
`.kamn/demo/run-55450-1783755996034/` and produced report status `GO` plus a
canonical verifier `PASS`. Its escrow `escrow-local-ab2190a8436d1bd7` settled
exactly one 1,000,000-lamport transfer
from `2FjUiacAXtokhA8YzGiyfVEdu5D9LxKFhjptJLrz4V9T` to
`FV5LvudLjZQGCrPwXUY2JaVr26sQE15K25BGvsKWvyFe`. Solana devnet finalized
signature
`3qZigXwYm3msVGbv9h7ShujMNRxKidDxoXZ7MKExZN8Js7MvU8dfsPpWbTLoZcpNKVQp9DY3oijhojRJoTf2QB8W`;
the persisted intent is `confirmed`, and the report carries the same signature,
commitment, recipient, amount, and before/after balances.

The earlier successful report at `.kamn/demo/run-68754-1783753805208/`
predates cryptographic persisted-transaction validation and escrow-specific
memo binding. It remains valid evidence for the earlier transaction shape but
is superseded by the final rehearsal above and is not the completion artifact
for the hardened implementation.

Two earlier NO-GO rehearsals exposed the confirmation-visibility crash window.
Both transfers finalized on devnet but their local escrow state remained
non-released (`prepared` before the classification fix, then `ambiguous` before
the retry horizon was extended). Their preserved signatures are
`VpALZZJjfUBzLWr1rkm7YDAc4o3c6BzDbJZXxZsmgK9geUoPE7TA87UgQSsTgybEvNu6ciEW2GeZtif6oDFCUrD`
and
`4fBZ5NiFTQi7vhFvVQ3gDdzu3zC3g15W9ThLrgALQ5ySdNW3y161LubYBzMQgQK7FwbNkHP54yza9H8bhQAEBGEw`.
They are not counted as successful MVP reports. This evidence drove the final
post-submit ambiguity classification and bounded same-signature reconciliation
behavior; the artifacts remain available for audit and must not be relabeled as
successful escrows.
