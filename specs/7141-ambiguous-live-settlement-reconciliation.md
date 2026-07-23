# Issue #7141: Reconcile Ambiguous Live Settlement Before Retry

## Objective

Prevent a lost or truncated release response from making an accepted Solana transfer
look unsubmitted in durable service state. Persist one submission attempt immediately
before broadcast, reconcile the deterministic expected signature before any retry, and
give the canonical MCP SDK enough time to receive finalized settlement responses.

## Inputs And Outputs

### Inputs

- A persisted `prepared` settlement intent containing the signed transaction and expected
  signature.
- A live Solana settlement configuration using finalized devnet confirmation.
- A canonical Pi/MCP escrow-release request with a stable idempotency key.
- The canonical supervisor RPC timeout budget.

### Outputs

- A durable `submitted` settlement intent with `submission_attempt_count == 1` before the
  signed transaction reaches the Solana broadcast call.
- A retry path that queries the expected signature before considering resubmission.
- One confirmed/released durable result with the expected signature and receipt hash.
- A confirmed/released response carrying the durable `escrow:release-authorize`
  receipt identity alongside the separate Solana settlement metadata.
- A canonical MCP SDK request timeout that is no shorter than the supervisor RPC budget.

## Boundaries And Non-Goals

- Do not change transaction construction, deterministic signatures, finalized RPC
  verification, the existing `escrow:release-authorize` authority action, or
  idempotency keys.
- Do not add dependencies, chains, mainnet behavior, production custody, or a second
  settlement path.
- Do not retry the live transfer observed while diagnosing #7141.
- Do not weaken failure handling when the signature is absent, failed, expired, or bound
  to different settlement terms.
- Do not change the SDK's general two-second default; configure the longer timeout only
  for the canonical agent-transaction child environment.

## Failure Modes

- The process exits after broadcast but before confirmation or final persistence.
- The client times out and closes the HTTP connection while Solana confirmation is still
  running.
- A retry sees `submitted` durable state and the expected signature is already finalized.
- A retry sees no signature and the persisted blockhash is valid or expired.
- Persistence fails immediately before broadcast; the transaction must not be submitted.
- A callback or adapter failure is translated into a structured hard failure without a
  second submission.
- The Pi extension strips the SDK timeout while constructing the MCP child environment,
  causing the MCP server to retain the two-second SDK default.
- The Pi extension's MCP request timer remains at ten seconds even when the MCP SDK is
  configured with the longer canonical timeout.
- A finalized or replayed release response omits the durable authorization receipt ID,
  digest, or action and therefore cannot satisfy the SDK authority contract.
- A released escrow has no durable `escrow:release-authorize` receipt; the response must
  hard-fail instead of fabricating client-local evidence.
- Conflicting actor, idempotency key, signature, recipient, amount, network, or evidence.

## Acceptance Criteria

- [ ] The live adapter invokes a durable write-ahead callback immediately before its one
  broadcast call.
- [ ] The callback persists `state == "submitted"` and exactly one submission attempt.
- [ ] Callback persistence failure prevents broadcast and hard-fails the release.
- [ ] Ambiguous, successful, and reconciled outcomes retain exactly one durable attempt.
- [ ] Retry queries the expected signature before blockhash validation or resubmission and
  never broadcasts when that signature is already confirmed.
- [ ] Expired transactions fail closed without a new broadcast.
- [ ] The canonical Pi/MCP child environment sets the SDK timeout from the supervisor RPC
  timeout budget without changing the SDK-wide default.
- [ ] The Pi extension forwards that timeout to the MCP server while continuing to strip
  credentials, custody configuration, and unrelated parent environment values.
- [ ] The persistent MCP session request timer is no shorter than the forwarded SDK
  timeout, so the extension does not terminate an in-flight settlement first.
- [ ] Successful and idempotently replayed live release responses include the same
  durable `escrow:release-authorize` receipt ID and digest while reporting `released`.
- [ ] Solana signature, receipt hash, network, and commitment remain separate settlement
  metadata; the authorization digest is not relabeled as a settlement digest.
- [ ] Regression tests cover write-ahead ordering, callback failure, ambiguous retry,
  confirmed reconciliation, and canonical timeout propagation.
- [ ] The fresh-checkout canonical demo and standalone verifier pass with one transfer.
- [ ] Formatting, strict Clippy, focused tests, `make check`, and relevant CI pass.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch.rs`
- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch/transport.rs`
- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch/test_support.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops/settlement_intent.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/update_routes/state_routes_release/live_settlement.rs`
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests/task_escrow_persistence_contract_tests/`
- `crates/kamn-e2e-harness/src/agent_transaction_evidence.rs`
- Focused harness configuration tests.
- `.pi/extensions/kamn-mvp/mcp-session.ts`
- `.pi/extensions/kamn-mvp/mcp-session.test.ts`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops/settlement.rs`

## Error Semantics

- Write-ahead persistence failure returns the existing service persistence error and does
  not call the settlement transport.
- Confirmed expected signatures return canonical settlement evidence without broadcast.
- Ambiguous submission or confirmation returns `SETTLEMENT_OUTCOME_AMBIGUOUS` after the
  durable attempt marker exists.
- Expired transactions return `SETTLEMENT_TRANSACTION_EXPIRED` without broadcast.
- Conflicts retain `SETTLEMENT_INTENT_CONFLICT` or `SETTLEMENT_AGREEMENT_MISMATCH`.
- MCP and SDK boundaries return a complete structured error; no silent fallback or local
  authority receipt is allowed.
- A released escrow missing its durable release-authorization receipt returns
  `ESCROW_RECEIPT_MISSING`; no receipt is synthesized during response serialization.

## Test Plan

### RED

- Require the test adapter to observe durable `submitted` state and attempt count one at
  its simulated broadcast boundary.
- Inject write-ahead callback failure and prove the adapter submission count remains zero.
- Require the canonical child environment to propagate an SDK timeout derived from the
  configured supervisor RPC timeout.
- Require the Pi extension MCP spawn environment to retain that timeout without widening
  its credential and custody allowlist.
- Require finalized and replayed live releases to expose one stable durable authorization
  receipt ID, digest, and action in the SDK response shape.

### GREEN

- Add a one-shot pre-broadcast callback to settlement submission/reconciliation.
- Persist the submitted state and attempt count through the message store callback.
- Thread the callback through real and test dispatch without changing reconciliation
  ordering.
- Add canonical SDK timeout propagation to the Pi/MCP runtime environment.
- Resolve the already-persisted release-authorization receipt when serializing finalized
  and replayed live release responses.

### REFACTOR

- Keep transport, durable intent transitions, route error translation, and supervisor
  environment calculation in their existing single-purpose modules.
- Keep functions at or below 25 lines and touched files at or below 200 lines.
- Remove duplicate attempt mutation while preserving monotonic state transitions.

### INTEGRATION

- Run focused settlement transport, persistence, ambiguous retry, and agent-transaction
  environment contracts.
- Run formatting, strict Clippy, touched Rust size policy, and `make check`.
- From a detached fresh `origin/main` worktree, run the canonical demo followed by the
  standalone verifier and reconcile exactly one finalized transfer.
