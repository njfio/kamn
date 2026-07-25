# 7173 Live Settlement Authority Parity

## Objective

Prove that SDK, CLI, and MCP settlement attempts preserve one escrow,
idempotency identity, and complete bridge-authorized settlement receipt while
causing exactly one economic submission.

## Inputs / Outputs

Inputs:
- one escrow ID and one idempotency key shared by all three drivers
- SDK, CLI, and MCP release responses
- the expected actor DID, resource ID, and economic terms
- authoritative bridge and settlement receipt fields
- a settlement-submission counter
- optional live payer and recipient balances plus finalized RPC evidence

Outputs:
- one byte-stable normalized authoritative settlement per driver
- a parity report containing the shared identity and submission count
- structured authority or replay errors for every rejected attempt
- an ignored-live evidence report when funded execution is explicitly enabled

## Boundaries / Non-Goals

- Do not redesign the public settlement authority schema.
- Do not create three independent escrows and call them parity evidence.
- Do not accept `escrow_id` and `state` without complete authority.
- Do not add dependencies, modify CI, or spend funds automatically.
- Deterministic integration evidence does not prove a funded devnet transfer.
- Do not promote the external x402 probe while peer receipt fields are absent.

## Failure Modes

- A driver changes the escrow ID or idempotency key.
- A response omits or partially populates authoritative settlement.
- Receipt digests, actor, resource, economics, terms, or receipt chain differ.
- A transaction signature or finalized slot is missing.
- The same idempotency key is replayed with different authority.
- A receipt from another escrow or transaction is reused.
- More than one settlement submission occurs across the three attempts.
- Live balance movement, RPC confirmation, or submission count is absent.

## Acceptance Criteria

- [x] The harness exports a shared three-driver settlement parity verifier.
- [x] SDK, CLI, and MCP attempts bind to one escrow ID and idempotency key.
- [x] All drivers yield byte-identical normalized authoritative settlement.
- [x] Complete bridge, receipt, actor, resource, economic, transaction, slot,
      terms, and receipt-chain fields are validated before parity passes.
- [x] Missing, partial, tampered, cross-resource, and conflicting replay
      evidence fails closed with existing structured error taxonomy.
- [x] A deterministic integration test exercises real SDK, CLI, and MCP
      dispatch entrypoints against one stateful service boundary.
- [x] The stateful boundary records exactly one settlement submission.
- [ ] An ignored-live path records before/after balances, finalized RPC
      evidence, and exactly one submission for the same transfer identity.
- [x] Documentation distinguishes deterministic contract evidence from an
      actually executed funded proof.
- [x] Focused tests, formatting, and Clippy pass.
- [ ] The serial repository suite passes on a host with GNU `timeout`.

## Files To Touch

- `specs/7173-live-settlement-authority-parity.md`
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/settlement_authority_parity.rs`
- `crates/kamn-e2e-harness/src/settlement_authority_parity_validate.rs`
- `crates/kamn-e2e-harness/tests/settlement_authority_parity_contract.rs`
- `crates/kamn-e2e-harness/tests/authoritative_settlement_entrypoint_parity.rs`
- `crates/kamn-e2e-harness/tests/live_s05_settlement_authority_parity.rs`
- `docs/validation/live-s05-settlement-authority-parity.md`

## Error Semantics

Authority validation precedes parity or settlement-evidence promotion.
Missing or inconsistent authority returns `PI_SERVICE_AUTHORITY_MISMATCH`.
Malformed receipt-chain evidence returns `RECEIPT_CHAIN_INVALID`. Reusing the
same idempotency key with different authority returns the existing replay
error. Every error includes the driver and mismatched field in context.
Interior validation returns typed errors and does not log.

## Test Plan

Red:
- Public API and complete three-driver parity contract.
- Missing and partial authority mutation table.
- Receipt, actor, resource, economic, terms, and chain mutation table.
- Same-key/different-authority and cross-resource replay table.
- Exactly-once stateful entrypoint integration contract.

Green:
- Add the minimum shared attempt, report, and structured error types.
- Validate each normalized authority field and byte equality.
- Count one stateful settlement submission across the three real entrypoints.

Refactor:
- Separate identity, authority, and exactly-once validation.
- Centralize driver-specific response normalization.
- Keep every file under 200 lines and every function under 25 lines.

Integration:
- Replace the fixed-response parity test with a stateful service boundary.
- Add an ignored-live coordinator that requires explicit live environment.
- Persist live balances, RPC confirmation, and submission-count evidence.
- Run focused tests, formatting, Clippy, and the serial repository suite.

## Verification Evidence

- `cargo test -p kamn-e2e-harness --test settlement_authority_parity_contract
  --test authoritative_settlement_entrypoint_parity
  --test live_s05_settlement_authority_parity`: five passed; the funded live
  capture remained ignored because no explicit capture was supplied.
- `cargo fmt --all -- --check`: passed.
- `cargo clippy -p kamn-e2e-harness --all-targets -- -D warnings`: passed.
- `RUST_TEST_THREADS=1 make test`: reached
  `shell_test_surface_migration_wave1`; 17 tests passed and three wrapper
  parity tests could not start because this macOS host has no GNU `timeout`.
  No #7173 assertion failed.

## Deviations

The deterministic acceptance surface is complete. A funded capture is still
`NOT_EXECUTED`, so this change does not claim live economic movement. The full
serial suite is also not recorded as passed until it runs on a host providing
GNU `timeout`; the observed failure is an environment prerequisite, not a
waiver or a weakened test.
