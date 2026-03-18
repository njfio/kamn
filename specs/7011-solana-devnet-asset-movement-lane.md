# 7011 Solana Devnet Asset-Movement Lane

## Objective
Implement one honest bounded Solana devnet asset-movement lane on current `main`. The lane must move beyond live Solana observation and evidence-coupled settlement semantics by submitting one real Solana devnet transaction from a public KAMN runtime path, persisting the resulting transaction signature, and reconciling terminal escrow settlement state only after that same transaction reaches the configured Solana commitment.

## Inputs/Outputs
- Inputs:
  - one existing public escrow release path on the service-api runtime surface
  - one Solana devnet RPC URL
  - one bounded devnet signer/keypair and recipient address used only for this proof lane
  - one escrow identifier that must map to one on-chain submission attempt
- Outputs:
  - one real Solana devnet transaction submission from the KAMN runtime
  - one persisted transaction linkage containing the real Solana signature and reconciliation metadata
  - one operator-facing validation runbook under `docs/validation/`
  - one hard-fail docs contract guarding the runbook and proof-index entry
  - one integration or e2e proof that exercises the real path, including restart-visible reconciliation

## Boundaries/Non-goals
- Do not claim broad production readiness.
- Do not claim generalized multi-chain settlement.
- Do not claim Byzantine-safe dispute resolution.
- Do not widen this issue into a full custody architecture.
- Do not claim bridge finality beyond one bounded Solana devnet settlement lane.
- Do not accept another synthetic receipt-hash proof that never submits an on-chain transaction.

## Failure modes
- the runtime still derives synthetic receipt material from observed slots without submitting a real Solana transaction
- terminal settlement state is emitted before the submitted Solana signature reaches the configured commitment
- the transaction signature is not persisted or cannot be reconciled after restart
- repeated release attempts submit duplicate transfers for the same escrow
- startup accepts an enabled live settlement lane with invalid signer, recipient, or RPC config
- the runbook overstates the claim as broad production settlement, bridge settlement, or generalized Solana support

## Acceptance criteria (testable booleans)
- [ ] one public KAMN runtime path submits one real bounded Solana devnet transaction for escrow settlement semantics
- [ ] the runtime persists the real Solana transaction signature plus reconciliation metadata on the public settlement surface
- [ ] terminal settlement state is emitted only after the configured Solana commitment is observed for that same submitted signature
- [ ] one restart proof shows that the same persisted Solana signature remains queryable and linked to the same escrow after process restart
- [ ] repeated release for the same escrow is idempotent and does not create a second on-chain transfer
- [ ] one validation runbook exists at `docs/validation/` with explicit `What This Proves` and `What This Does Not Prove` sections
- [ ] one hard-fail docs contract fails when the runbook or proof-index marker is missing or overstates the claim
- [ ] `docs/validation/current-proven-runtime-slices.md` links the new runbook

## Files to touch
- `specs/7011-solana-devnet-asset-movement-lane.md`
- `docs/validation/*solana*asset*movement*.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/*solana*asset*movement*contract*.rs`
- `crates/kamn-node/src/service_api_endpoint/live_settlement_dispatch.rs`
- `crates/kamn-node/src/service_api_endpoint/middleware_impl/http_routes/mutations/update_routes/state_routes.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/models.rs`
- one bounded integration or e2e proof under `crates/kamn-node/src/main_tests/` or `crates/kamn-e2e-harness/tests/`
- only if strictly necessary, one small runtime helper under `scripts/runtime/`

## Error semantics
- startup must fail loud when the Solana devnet asset-movement lane is enabled with invalid or missing RPC URL, signer material, recipient address, amount, or commitment configuration
- the release path must fail hard when transaction construction, submission, lookup, or commitment reconciliation fails
- no silent fallback to synthetic slot-derived receipt hashes is allowed when the live asset-movement lane is explicitly selected
- repeated release for an already-submitted escrow must return the existing persisted linkage instead of creating a second transfer
- boundary handlers may translate typed failures into response envelopes, but interior code must return structured errors and preserve cause

## Test plan
1. Red:
   - add a docs contract for the missing runbook and proof-index marker
   - add one integration or e2e proof that expects a real Solana transaction signature on the settlement path and restart-visible reconciliation for that same signature
   - add one idempotency proof that repeated release does not create a second distinct Solana signature
   - confirm the new tests fail on current `main`
2. Green:
   - implement the smallest honest Solana devnet transfer submission and confirmation path from the service-api escrow release lane
   - persist the signature and reconciliation metadata
   - publish the runbook and proof-index entry
3. Refactor:
   - split helpers at logical seams so no touched file exceeds repo limits
   - remove duplication between live Solana observation helpers and the new settlement submission/reconciliation path where possible
4. Verification:
   - docs contract
   - real integration or e2e proof
   - restart reconciliation proof
   - idempotency proof
   - touched-Rust policy gate

## Dependency and implementation guard
- Prefer existing repo surfaces and the existing Solana proof lane first.
- If a new Solana client dependency or external CLI dependency is required, stop after Phase 2 and ask before implementation.

## Notes
- Current `main` already proves live Solana devnet observation, live bridge evidence derivation, and one bounded external-chain-backed settlement lane.
- This issue exists to replace slot-derived synthetic settlement linkage with one real Solana devnet transaction path, or fail loudly if the runtime cannot support it honestly.
