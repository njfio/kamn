# Issue 7123: Fresh-Checkout Evaluator Rehearsal

## Objective

Prove from a fresh clone of the merged remote head that an evaluator can run
the canonical three-agent KAMN MVP, independently verify its finalized Solana
devnet transfer, and inspect a secret-safe release-candidate evidence record.

## Inputs And Outputs

Inputs:

- Merged `origin/main` commit `6428f9f88a9aab4af248753c994070042416cebe`.
- README and `docs/validation/mvp-evaluator-demo.md` instructions only.
- Existing local Pi OAuth and funded devnet configuration supplied outside git.

Outputs:

- A new clean-clone canonical run and independently verified report.
- A secret-safe transcript outside git for operator audit.
- A tracked evidence summary containing only public identifiers, decisions,
  digests, counts, claim boundaries, and the Solana Explorer link.
- A fail-closed Rust contract for the tracked evidence shape.

## Boundaries And Non-Goals

- Do not change runtime, settlement collection, Pi/MCP authority, protocol, API,
  dependency, or release behavior.
- Do not copy the development checkout's `target/`, `.kamn/`, generated files,
  running processes, or shell history into the clean clone.
- Do not commit environment files, key paths, key contents, credentials, raw
  proof bundles, or the full evaluator transcript.
- Do not claim production, mainnet, custody, generalized exchange, or real
  economic value.
- A `command-override` result may characterize the current baseline but does
  not satisfy the later direct-receipt issue.

## Failure Modes

- Clone provenance does not match the merged remote head.
- The canonical command needs an undocumented intervention.
- Pi, RPC, faucet, or devnet infrastructure is unavailable.
- The standalone verifier fails after child shutdown.
- The signature is absent, non-finalized, copied from an older run, or does not
  match recipient, amount, slot, and balance evidence.
- Retry or recovery submits a second transfer.
- Agent identities are not distinct or Agent C exposes private fields.
- The tracked evidence contains a private path, credential, or secret value.

## Error Semantics

- Any unmet live prerequisite or proof mismatch records `NO-GO` with explicit
  reason codes; it must not be rewritten as success.
- Contract failures identify the missing or forbidden marker without printing
  matched sensitive content.
- Only a new finalized transaction observed through independent RPC can support
  the devnet settlement result.

## Acceptance Criteria

- [ ] A temporary clone starts from the recorded merged `origin/main` SHA with
  no inherited build or proof state.
- [ ] `make demo-agent-transaction` produces a new required-devnet run.
- [ ] The standalone verifier returns `GO` after all child processes exit.
- [ ] Independent RPC confirms signature, finalized commitment, slot,
  recipient, 1,000,000 lamports, and exact recipient balance movement.
- [ ] Evidence shows three distinct authenticated Pi actors and the complete
  task, authorization, escrow funding, completion, and release lifecycle.
- [ ] Evidence shows exactly one transfer and retry/recovery idempotency.
- [ ] Agent A and B participant-private projections differ from Agent C's
  restricted-public projection; Agent C has zero participant-private fields.
- [ ] Durable receipts, relay, websocket visibility, audit export, proof report,
  and a devnet Explorer link are recorded.
- [ ] The tracked evidence passes secret/path and bounded-claim contracts.

## Files To Touch

- `specs/7123-fresh-checkout-evaluator-rehearsal.md`
- `docs/validation/evidence/7123-fresh-checkout-evaluator-rehearsal.md`
- `crates/kamn-e2e-harness/tests/fresh_checkout_evaluator_evidence_contract.rs`
- `fixtures/ci/test_file_size_policy_baseline.env`

## Test Plan

### RED

Require the evidence document, fresh-clone provenance, live command/verifier
results, independent RPC fields, actor/view boundaries, idempotency, durable
proof surfaces, bounded claims, and secret-safe content.

### GREEN

Perform the rehearsal in a temporary clone and publish only the allowed summary
fields from the new run. Record `NO-GO` honestly if an external dependency
blocks the live path.

### REFACTOR

Keep the contract below 200 lines, functions below 25 lines, and marker checks
table-driven. Keep the evidence concise and avoid duplicating the runbook.

### INTEGRATION

```bash
cargo test -p kamn-e2e-harness --test fresh_checkout_evaluator_evidence_contract
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
make check
make test
make pre-push
```

## Rollback

- Revert the issue branch if the evidence cannot be published without leaking
  local configuration.
- Preserve any valid `NO-GO` diagnosis outside success claims.
- Never reuse an older signature or weaken privacy, idempotency, or verifier
  semantics to obtain `GO`.
