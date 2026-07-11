# Bind Escrow Funding And Release To The Task Agreement

## Objective

Turn the existing generic escrow record into a durable, local-only release
authorization contract for one #7090 task agreement. Prove that only the
authenticated task creator can fund bounded devnet-denominated escrow terms and
that release becomes eligible only after the assigned provider completes the
same task with evidence.

## Inputs/Outputs

Funding input is JSON containing `task_id`, `transaction_id`, `beneficiary_did`,
`amount_lamports`, `network`, `terms_digest`, `release_authority_did`,
`release_policy`, and `idempotency_key`. The authenticated actor is the funder
and is never accepted from the body.

The MVP contract accepts only:

- `network`: `solana-devnet`
- `release_policy`: `task-completed`
- `amount_lamports`: an integer from 1 through 1_000_000
- `release_authority_did`: the authenticated task creator
- `beneficiary_did`: the task provider

Funding returns the escrow ID, task and transaction linkage, participants,
bounded amount, network, terms digest, policy, and `funded` state. Release input
contains `idempotency_key`; release returns the same immutable agreement,
`release-authorized` state, and a durable transition receipt ID.

## Boundaries/Non-goals

- Do not submit, sign, simulate, or claim a Solana transaction.
- Do not move assets, debit balances, or claim funded custody.
- Do not implement mainnet, another chain, disputes, refunds, mediation,
  partial release, expiry, or policy language.
- Do not accept funder identity, task actors, terms, or release authority from
  unverified headers or body identity claims.
- Do not change #7090 task lifecycle semantics.
- Do not add a generalized escrow engine or new dependency.

The `funded` and `release-authorized` states are local-only authorization proof.
Devnet submission and final settlement evidence belong to #7092.

## Escrow Contract

A new escrow durably stores:

- escrow ID and idempotency key
- task ID and transaction ID
- funder DID and beneficiary DID
- amount in lamports and `solana-devnet` network
- terms digest
- release authority DID and `task-completed` policy
- state

Funding requires an existing canonical task in `accepted` state. The task
creator must equal the authenticated funder and release authority. The task
provider must equal the beneficiary. Transaction ID and terms digest must match
the task exactly. No field can change after funding.

Release requires the same authenticated release authority, the linked task in
`completed` state, unchanged agreement fields, and persisted completion
evidence. A successful release changes only the local escrow state to
`release-authorized`; it does not populate settlement transaction metadata.

## Idempotency

- Repeating funding with the same actor, key, and canonical payload returns the
  existing escrow.
- Reusing a funding key with different task, transaction, participants, amount,
  network, terms, authority, or policy fails without mutation.
- Repeating release with the same actor, key, and escrow returns the existing
  receipt and state.
- Reusing a release key for another actor or escrow fails without mutation.
- A new key after release authorization returns an explicit state conflict and
  does not append another successful receipt.

## Receipts

Every successful funding or release transition appends one durable receipt with
receipt ID, redacted correlation join key, idempotency key, authenticated actor,
escrow ID, task ID, transaction ID, action, prior state, resulting state,
network, amount, terms digest, and release policy. Denied domain transitions
return structured errors and do not append a successful escrow receipt.

## Failure Modes

- Missing or malformed input: `ESCROW_AGREEMENT_INVALID` (400).
- Unknown task or escrow: existing route-not-found response (404).
- Non-canonical legacy task: `ESCROW_TASK_AGREEMENT_REQUIRED` (409).
- Funder is not task creator: `ESCROW_FUNDER_MISMATCH` (403).
- Beneficiary is not task provider: `ESCROW_BENEFICIARY_MISMATCH` (409).
- Transaction or task mismatch: `ESCROW_TRANSACTION_MISMATCH` (409).
- Terms mismatch: `ESCROW_TERMS_MISMATCH` (409).
- Amount outside the bounded range: `ESCROW_AMOUNT_INVALID` (400).
- Network is not `solana-devnet`: `ESCROW_NETWORK_INVALID` (400).
- Release authority mismatch: `ESCROW_RELEASE_AUTHORITY_MISMATCH` (403).
- Funding before task acceptance: `ESCROW_TASK_STATE_CONFLICT` (409).
- Release before task completion/evidence: `ESCROW_RELEASE_NOT_ELIGIBLE` (409).
- Conflicting retry: `ESCROW_IDEMPOTENCY_CONFLICT` (409).
- Persistence failure: existing state-persistence response (500).

## Acceptance Criteria

- [ ] Funding derives funder DID only from verified request context.
- [ ] Funding persists the complete immutable task, participant, amount,
      network, terms, authority, policy, and idempotency agreement.
- [ ] Funding requires an accepted canonical task and exact #7090 linkage.
- [ ] Only the task creator can fund and act as release authority.
- [ ] Only the task provider can be beneficiary.
- [ ] Release requires completed task state and completion evidence.
- [ ] Identical retries return the existing escrow or release receipt.
- [ ] Conflicting retries fail without state or receipt mutation.
- [ ] Successful transitions persist secret-free receipts.
- [ ] Escrow agreement, idempotency, and receipts survive restart.
- [ ] Legacy generic escrows remain readable but cannot use protected release.
- [ ] Signed HTTP integration proves task accept -> escrow fund -> task complete
      -> release authorization without settlement submission.
- [ ] Output and tests label the result local-only and do not claim custody,
      settlement, or asset movement.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint/models.rs`
- focused escrow models under `service_api_endpoint/message_store/`
- focused funding, validation, idempotency, and receipt modules under
  `message_store/store/task_escrow_ops/`
- escrow funding and release HTTP handlers
- task/escrow integration and restart contracts
- vertical-slice assertions only where canonical task ordering requires parity

## Error Semantics

Interior code returns a typed escrow lifecycle error. HTTP entrypoints map each
error once. Domain denials never invoke a live settlement adapter and never
write settlement metadata. Persistence errors hard-fail; no success response is
reported after a failed state write.

## Test Plan

RED:

- Funding currently accepts only `task_id` and immediately reports `funded`.
- Funding ignores actor, task state, participants, transaction, terms, amount,
  network, authority, policy, and idempotency.
- Release currently accepts no actor or task eligibility and reports released.
- No escrow transition receipt exists.

GREEN:

- Add minimal serde-defaulted escrow agreement and receipt fields.
- Parse canonical payloads and pass verified actor/correlation into store calls.
- Validate exact task agreement and state before mutation.
- Implement file-backed idempotency and typed route errors.

REFACTOR:

- Separate payload validation, task linkage, retry matching, receipts, and
  responses into focused modules no larger than 200 lines.
- Remove generic mutation helpers that bypass the protected contract.
- Keep all functions at or below 25 lines and strict clippy clean.

INTEGRATION:

- Exercise signed task create -> provider accept -> fund -> provider complete ->
  release-authorized across restart.
- Inspect persisted agreement, receipts, completion evidence, and empty
  settlement metadata.
- Prove wrong actor, early state, mismatched agreement, and retry conflicts.
- Run targeted escrow contracts, full `kamn-node`, formatting, strict workspace
  clippy, and `make check`.
