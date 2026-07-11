# Bind Task Lifecycle To Creator And Assigned Provider

## Objective

Turn the existing generic task state into one durable agent transaction between
an authenticated creator and one named provider. Bind every transition to an
immutable transaction ID and terms digest, enforce the submitted -> accepted ->
completed state machine, and emit durable transition receipts.

## Inputs/Outputs

Task creation input is JSON containing:

- `provider_did`: one registered provider DID.
- `transaction_id`: caller-supplied stable transaction identifier.
- `terms_digest`: lowercase 64-character hex digest of canonical terms.
- `idempotency_key`: stable creation retry key.
- Optional `task_type` and `description` discovery metadata.
- Optional legacy `creator`; when present it must equal the authenticated actor.

The creator DID always comes from the verified request context, never from the
body. Creation returns the task ID, transaction ID, creator DID, provider DID,
terms digest, and `submitted` state.

Task acceptance input is JSON containing `idempotency_key`. Completion input is
JSON containing `idempotency_key` and a lowercase 64-character hex
`completion_evidence_digest`. Transition responses include the same immutable
task agreement fields, resulting state, and transition receipt ID.

## Task Contract

New tasks durably store:

- task ID
- transaction ID
- creator DID
- provider DID
- terms digest
- state
- optional completion evidence digest
- create idempotency key
- optional task type and description

Creation atomically issues exact active server-owned grants for the provider to
`task:accept` and `task:complete` on `task:{task_id}`. It also issues exact
`task:read` grants for creator and provider. Grant IDs are deterministic from
task ID, actor, action, and role, so retrying creation cannot duplicate
authority.

The legal state machine is:

1. Creator creates `submitted` task naming exactly one provider.
2. Named provider accepts `submitted` -> `accepted`.
3. Same provider completes `accepted` -> `completed` with evidence digest.

No actor can reassign the provider or change transaction ID or terms digest.

## Idempotency

- Repeating create with the same creator, idempotency key, and canonical payload
  returns the existing task.
- Reusing a create idempotency key with different agreement fields fails.
- Repeating a transition with the same actor, action, idempotency key, and
  canonical payload returns the existing receipt and resulting task state.
- Reusing a transition idempotency key with different task, actor, action, or
  completion digest fails without mutation.
- A new idempotency key for an already-applied transition returns an explicit
  state conflict; it does not manufacture a second receipt.

## Transition Receipts

Every successful accept or complete transition appends one durable receipt with:

- receipt ID
- redacted correlation join key
- idempotency key
- actor DID
- task ID
- transaction ID
- action
- prior state
- resulting state
- terms digest
- optional completion evidence digest

Authorization denials continue to use #7089 authorization receipts. Domain
state/order/idempotency denials return structured errors and do not append a
successful transition receipt.

## Boundaries/Non-goals

- Do not implement escrow funding, release, Solana submission, or value movement.
- Do not implement bidding, reassignment, delegation, teams, or a marketplace.
- Do not trust body `creator`, provider headers, capabilities, or scope as actor
  identity or authority.
- Do not add a public grant administration API.
- Do not add a generalized workflow engine.
- Do not change message, content, bridge, websocket transport, or unrelated
  authorization behavior.
- Do not claim that task completion settles payment.

## Failure Modes

- Missing or invalid provider DID: `TASK_PROVIDER_INVALID`.
- Provider is not registered: `TASK_PROVIDER_NOT_REGISTERED`.
- Body creator differs from authenticated actor: `TASK_CREATOR_MISMATCH`.
- Missing/invalid transaction ID or terms digest: `TASK_AGREEMENT_INVALID`.
- Authenticated actor is not assigned provider: `TASK_PROVIDER_MISMATCH`.
- Accept outside `submitted`: `TASK_STATE_CONFLICT`.
- Complete outside `accepted`: `TASK_STATE_CONFLICT`.
- Missing/invalid completion evidence digest: `TASK_COMPLETION_EVIDENCE_INVALID`.
- Conflicting idempotency-key reuse: `TASK_IDEMPOTENCY_CONFLICT`.
- Legacy task missing agreement fields: `TASK_AGREEMENT_MIGRATION_REQUIRED`.
- Persistence or grant issuance failure: hard state persistence error; no partial
  task, grant, transition, or receipt may be reported as successful.

## Acceptance Criteria

- [x] Creation derives creator DID only from verified request context.
- [x] A new task persists one creator, one provider, transaction ID, terms
      digest, create idempotency key, and submitted state.
- [x] Creation requires the named provider to be durably registered.
- [x] Creation issues deterministic exact read/accept/complete grants needed by
      the real #7089 middleware path.
- [x] Only the assigned provider can accept and complete.
- [x] Creator, unrelated actor, and verifier cannot perform provider transitions.
- [x] Submitted tasks cannot complete; accepted tasks cannot accept again under
      a new idempotency key.
- [x] Completion requires and persists a valid evidence digest.
- [x] Agreement identity cannot change after creation.
- [x] Identical retries return the existing task or transition receipt.
- [x] Conflicting retries fail without state or receipt mutation.
- [x] Successful transitions persist one secret-free receipt with agreement and
      state linkage.
- [x] Task, grants, idempotency, and receipts survive restart.
- [x] Legacy snapshots load, but legacy incomplete tasks fail closed on protected
      transitions with an explicit migration error.
- [x] A live HTTP integration test proves authenticated create -> provider accept
      -> provider complete without mocking authorization or the message store.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint/models.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/models.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops.rs`
- focused task store/parser/receipt modules under that directory
- task create and transition HTTP route handlers
- #7089 grant validation/issuance store boundary
- task/escrow service API integration tests
- task response models in SDK/agent/MCP/CLI only where compilation or real path
  parity requires them

No new dependency is required.

## Persistence And Compatibility

- New snapshots use schema `kamn.runtime.service-api-message-store.v4`.
- New task agreement and receipt collections are serde-defaulted so v3
  snapshots load.
- Existing incomplete task records remain readable for status/audit purposes but
  protected transitions fail with `TASK_AGREEMENT_MIGRATION_REQUIRED`.
- Creation, lifecycle grants, task mutation, and transition receipt persistence
  occur under the existing message-store lock and one atomic state-file write.
- No migration silently invents creator, provider, transaction, terms, or
  evidence values.

## Error Semantics

Malformed task inputs return HTTP 400 in the standard JSON error envelope with
the stable task reason code. Wrong actor or missing lifecycle grant returns HTTP
403. Illegal state and idempotency conflicts return HTTP 409. Missing task
returns HTTP 404. Persistence errors return HTTP 500 with the existing state
persistence reason code.

Interior code returns typed task lifecycle errors. HTTP entrypoints map each
error once and log once. No fallback or partial success is permitted.

## Test Plan

RED:

- Task creation currently accepts no provider, transaction, terms, or idempotency
  contract and trusts optional body creator metadata.
- Any granted actor can transition any task because store transitions receive no
  actor identity.
- Task can currently move directly from submitted to completed.
- Completion currently accepts no evidence digest.
- No lifecycle grant issuance, idempotent retry, or transition receipt exists.

GREEN:

- Add the minimal v4 task agreement, receipt, and idempotency models.
- Parse and validate canonical create/transition payloads.
- Pass verified actor and redacted correlation join key into store operations.
- Enforce provider/state invariants and issue exact lifecycle grants.
- Return structured lifecycle errors from real HTTP routes.

REFACTOR:

- Separate payload validation, state transition, receipt creation, and grant
  issuance into focused modules.
- Keep files at or below 200 lines and functions at or below 25 lines.
- Remove duplicate task resource/action parsing and preserve fail-closed errors.

INTEGRATION:

- Exercise create -> accept -> restart -> complete through the live HTTP server.
- Inspect persisted agreement, exact grants, receipts, and completion evidence.
- Prove wrong actor, illegal order, duplicate retry, and conflicting retry paths.
- Run targeted node/SDK/MCP/CLI task tests, full node tests, strict clippy,
  formatting, and `make check`.
