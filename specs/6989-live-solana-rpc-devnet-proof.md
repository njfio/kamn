# 6989-live-solana-rpc-devnet-proof

## Objective
Prove one bounded live Solana RPC/devnet-backed proof lane on current `main` by querying public Solana `devnet` JSON-RPC for live commitment evidence, validating that evidence through a deterministic runner/validator/policy lane, and driving the observed commitment labels through KAMN's public `normalize_cross_chain_receipt(...)` surface. The proof must remain explicit about what it does not prove.

## Inputs/Outputs
- Inputs:
  - public Solana devnet JSON-RPC endpoint
  - existing public receipt normalization surface in `kamn_bridges` / `kamn_core`
  - current runtime proof index in `docs/validation/current-proven-runtime-slices.md`
- Outputs:
  - one executable live proof runner under `scripts/runtime/`
  - one validator and one policy checker under `scripts/runtime/`
  - one bounded runbook under `docs/validation/`
  - one live-gated Rust integration test that normalizes observed Solana labels through the public receipt surface
  - one hard-fail docs contract binding the runbook and proof index to the live lane

## Boundaries/Non-goals
- Do not claim live bridge settlement.
- Do not claim live message relay or service-api delivery over Solana.
- Do not claim live external bridge finality beyond live Solana RPC commitment observation plus normalization through the public receipt surface.
- Do not add a Solana SDK dependency unless the existing toolchain proves insufficient.
- Do not weaken or replace the existing bounded Solana proofs already on `main`.

## Failure modes
- The runner succeeds without making a real Solana devnet JSON-RPC request.
- The validator accepts malformed or incomplete live evidence.
- The policy checker returns GO when required commitment observations or normalization evidence are missing.
- The runbook overstates the proof as bridge settlement, production readiness, or live KAMN Solana relay.
- The live-gated Rust test does not actually consume the live report file and exercise `normalize_cross_chain_receipt(...)`.

## Acceptance criteria (testable booleans)
- [ ] A live proof runner exists under `scripts/runtime/` and performs real Solana devnet JSON-RPC requests for health/version/commitment evidence.
- [ ] A validator under `scripts/runtime/` fails closed on malformed, incomplete, or non-monotonic commitment evidence.
- [ ] A policy checker under `scripts/runtime/` returns GO only when the live evidence and normalization evidence are both present and valid.
- [ ] A runbook exists at `docs/validation/live-solana-devnet-proof-slice.md` with explicit "What This Slice Proves" and "What This Slice Does Not Prove" sections.
- [ ] A live-gated Rust integration test uses the generated report file and proves that live Solana labels normalize through the public `normalize_cross_chain_receipt(...)` surface.
- [ ] A hard-fail docs contract enforces the runbook markers and proof-index entry.
- [ ] `docs/validation/current-proven-runtime-slices.md` links the new slice with bounded wording.

## Files to touch
- `specs/6989-live-solana-rpc-devnet-proof.md`
- `scripts/runtime/run_live_solana_devnet_proof.py`
- `scripts/runtime/validate_live_solana_devnet_proof.py`
- `scripts/runtime/check_live_solana_devnet_proof_policy.py`
- `docs/validation/live-solana-devnet-proof-slice.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-core/tests/live_solana_devnet_receipt_normalization.rs`
- `crates/kamn-node/tests/live_solana_devnet_proof_slice_contract.rs`
- optionally `docs/review/corrected-audit-response-2026-03-14.md`

## Error semantics
- The runner must exit non-zero on network failure, JSON-RPC error response, invalid schema, or timeout.
- The validator must emit explicit machine-readable failure reasons and return non-zero on any invalid report shape or missing evidence.
- The policy checker must fail closed when live evidence is absent, when commitment monotonicity is violated, or when normalization evidence is missing.
- The Rust live test must skip only when the required env/report-file input is absent; once invoked by the runner it must fail loudly on invalid evidence.

## Test plan
- Phase 3 red tests: add a docs contract that fails because the live Solana runbook and proof-index markers do not exist yet.
- Green by adding the runner, validator, policy checker, bounded runbook, and proof-index entry.
- Add a live-gated Rust integration test that consumes the runner's report and normalizes live Solana commitment labels through `normalize_cross_chain_receipt(...)`.
- Run the runner, validator, policy checker, docs contract, and live-gated Rust test before publish.
- Run touched-Rust policy before publish.
