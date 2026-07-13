# Issue 7104: Agent Transaction Failure Matrix

## Objective

Prove that the canonical agent transaction remains at-most-once and
state-monotonic across restart, retry, concurrency, ambiguous Solana outcomes,
agent exit, and proof-generation failure. Close the residual stale-blockhash
gap without generating a replacement transaction signature.

## Inputs And Outputs

Inputs:

- Persisted task, authorization grant, escrow agreement, settlement intent,
  signed Solana devnet transaction, and runtime actor receipts.
- Two independently authorized release requests carrying the same release key.
- Solana signature status, prepared transaction blockhash validity, and bounded
  submit/confirmation outcomes.
- Canonical demo proof generation after settlement has already completed.

Outputs:

- One durable settlement signature and one monotonic terminal escrow state.
- Explicit `NO-GO` or structured settlement errors for unresolved paths.
- Restarted/retried execution that reconciles persisted identity before any
  network submission.
- Proof regeneration that reads persisted evidence and performs zero settlement
  submissions.

## Boundaries And Non-Goals

- Reuse the #7092 settlement intent, service API state file, Solana adapter,
  supervisor, and proof verifier surfaces.
- Do not add a workflow engine, database schema, replacement-signature flow,
  generalized recovery coordinator, or new dependency.
- Solana devnet only. Mainnet custody, production HA, multi-region consensus,
  refund/dispute handling, and economic-value claims remain out of scope.
- A never-landed expired transaction is terminal for this bounded intent. An
  operator-created replacement transaction is roadmap work and is not claimed.
- Test-only adapters may model RPC boundaries, but integration tests must use
  the real service API persistence and request paths.

## Failure Modes

- Restart loses task agreement, authorization grant, escrow, nonce history, or
  settlement intent.
- Retry submits before querying the persisted signature.
- A signature absent from RPC is blindly resubmitted after its recent blockhash
  has expired.
- Two authorized release requests create different signatures or more than one
  submission identity.
- Post-submit ambiguity or local persistence failure reports `released`.
- A confirmed settlement rolls back to `prepared`, `submitted`, `ambiguous`, or
  an unreleased escrow state.
- Proof generation or proof retry invokes the settlement adapter.
- Agent failure leaves child processes alive or emits a successful proof report.

## Error Semantics

- Signature status unknown while the persisted blockhash is still valid:
  `SETTLEMENT_OUTCOME_AMBIGUOUS` (503), with no release claim.
- Signature absent and the persisted transaction blockhash is expired:
  `SETTLEMENT_TRANSACTION_EXPIRED` (409), intent state `failed`, no resubmission,
  and no replacement signature.
- Confirmed on retry: reconcile the persisted signature and finalize once.
- Conflicting agreement/release identity: existing
  `SETTLEMENT_INTENT_CONFLICT` (409), without adapter invocation.
- Proof failure: `AGENT_TRANSACTION_PROOF_INVALID` and `NO-GO`; persisted
  settlement and escrow facts remain unchanged.
- Interior errors remain typed/stable strings; HTTP and supervisor entrypoints
  translate and log once without swallowing causes.

## Acceptance Criteria

- [ ] Restart after task creation preserves agreement and authorization state.
- [ ] Restart after escrow funding preserves the task-bound funded escrow.
- [ ] Restart from prepared, ambiguous, or submitted-equivalent intent queries
  the known signature before any same-transaction resubmission.
- [ ] Crash/ambiguity after transfer submission cannot create a second
  transaction identity or duplicate asset movement.
- [ ] Two independently authorized concurrent releases converge on one persisted
  signature and at most one network submission.
- [ ] Stale nonce/replay remains rejected after state reload.
- [ ] RPC timeout followed by later confirmation reconciles to released.
- [ ] A never-landed expired transaction becomes
  `SETTLEMENT_TRANSACTION_EXPIRED` without resubmission or replacement.
- [ ] Agent exit mid-flow emits `NO-GO` and cleans every child process.
- [ ] Proof generation failure and retry perform zero settlement submissions.
- [ ] Settled intent and escrow state cannot roll back on restart or retry.
- [ ] Focused failure matrix, funded devnet rehearsal, security review,
  formatting, strict clippy, `make check`, and `make test` pass.

## Files To Touch

- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch/`
  for status-first blockhash-validity classification at the RPC boundary.
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops/`
  only where monotonic transition enforcement is missing.
- Existing service API settlement persistence/reconciliation/concurrency tests
  under `crates/kamn-node/src/main_tests/service_api_endpoint_tests/`.
- Existing canonical transaction supervisor/proof tests under
  `crates/kamn-e2e-harness/tests/` when proof retry coverage is absent.
- This spec only; no schema, workflow, dependency, or README changes.

## Test Plan

### RED

1. Reload a funded task/escrow snapshot and assert agreement, grant, escrow, and
   replay nonce continuity.
2. Race two valid, distinct-nonce releases for the same release key and assert
   one signature rather than relying on replay rejection.
3. Model `signature_status=None` plus `blockhash_valid=false`; assert the current
   adapter resubmits or remains ambiguous, then require terminal expiration with
   submission count unchanged.
4. Model post-submit ambiguity, restart, and later confirmation; require one
   submission identity and monotonic finalization.
5. Fail proof writing after confirmed settlement, retry proof generation, and
   assert the settlement submission counter remains unchanged.
6. Reload settled state and attempt stale replay/transition; require rejection
   with the settled escrow and intent unchanged.

### GREEN

- Add the minimal RPC-boundary blockhash-validity classification for a persisted
  prepared transaction.
- Mark proven-expired, never-landed intents failed with the stable expiration
  code; never build a replacement transaction.
- Add only the transition guards and test hooks needed to make restart,
  concurrency, and proof-retry behavior observable.

### REFACTOR

- Keep preparation, status lookup, blockhash classification, submission,
  confirmation, and state transition responsibilities separate.
- Keep files below 200 lines and functions below 25 lines.
- Remove duplicate test setup by extending existing settlement fixtures.

### INTEGRATION

```bash
cargo test -p kamn-node agent_transaction_failure_matrix
cargo test -p kamn-node task_escrow_settlement
cargo test -p kamn-e2e-harness --test agent_transaction_demo_supervisor_contract
cargo test -p kamn-e2e-harness --test agent_transaction_demo_success_contract
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
make test
```

Run one bounded funded Solana devnet rehearsal using a fresh release key and the
existing configured payer/recipient. Record signature, recipient, lamports,
commitment, balances, persisted intent, canonical verifier output, and proof
report. If devnet RPC, faucet, or funding blocks the run, record explicit
`NO-GO`; do not claim live completion.

## Rollback

- Preserve RED, GREEN, REFACTOR, and integration commits separately.
- Keep a deterministic-test checkpoint before any funded rehearsal.
- Never delete `submitted`, `ambiguous`, confirmed, or expired settlement state.
- Before rollback or another live attempt, reconcile every persisted signature
  and retain any finalized devnet transaction as audit evidence.
