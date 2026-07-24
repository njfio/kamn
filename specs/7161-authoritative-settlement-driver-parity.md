# 7161 Authoritative Settlement Driver Parity

## Objective

Prove that SDK-direct, CLI-scripted, and MCP-agent settlement lanes expose and
validate the same bridge-authorized settlement receipt, including identical
transaction authority and deterministic rejection behavior.

## Inputs / Outputs

Inputs:
- bridge-authorized settlement receipt contract from #7163
- shared actor, task, escrow, economic terms, and idempotency key
- real SDK, CLI, and MCP S-05/S-13 entrypoints

Outputs:
- one driver-neutral normalized settlement observation
- additive receipt fields on SDK, CLI, agent-lib, and MCP surfaces
- per-driver positive and negative conformance results
- one secret-safe parity proof showing a single economic transfer

## Boundaries / Non-Goals

- Depends on #7160 and #7163.
- Do not introduce another settlement transaction.
- Do not claim new networks, assets, mainnet custody, performance, or production
  readiness.
- Do not equate matching terminal state strings with authority parity.
- Public model/output changes must be additive and preserve existing consumers.
- Do not add a dependency.

## Shared Receipt Contract

Every driver must expose and validate:
- bridge ID and bridge receipt ID/digest
- settlement receipt ID/digest
- action, resource ID, actor DID, and resulting state
- task ID, escrow ID, recipient, amount, asset, and network
- transaction signature, finalized commitment, and finalized slot
- receipt-chain commitment and terms digest
- idempotency key or durable operation identity

## Failure Modes

- A driver drops or rewrites an authoritative field.
- CLI prints only escrow ID/state or MCP accepts an unwrapped bridge mutation.
- SDK treats optional receipt authority as successful terminal settlement.
- Drivers normalize different digests for the same service response.
- Missing, partial, tampered, reordered, replayed, or cross-resource receipts
  produce inconsistent outcomes across drivers.
- Repeating through another driver submits a second transfer.
- Proof artifacts retain secrets or driver-private payloads.

## Acceptance Criteria

- [ ] SDK, CLI, and MCP expose every shared receipt field without inventing
      driver-local authority.
- [ ] MCP bridge mutations are service-authority wrapped and validated.
- [ ] CLI emits structured receipt authority and accepts an explicit durable
      idempotency identity.
- [ ] SDK treats required terminal settlement authority as non-optional.
- [ ] Equivalent service evidence yields byte-equivalent normalized authority
      fields across all three drivers.
- [ ] Each driver rejects missing service authority, partial fields, bad digest,
      wrong actor/resource/action/state, economic mismatch, reorder, and replay.
- [ ] A real parity integration uses one operation identity and proves the same
      receipt/signature with exactly one transfer across retry and restart.
- [ ] The runbook distinguishes adapter parity from generalized settlement.

## Files To Touch

Expected adapter surface:
- KAMN SDK service models and bridge/settlement client routes
- agent-lib bridge and release wrappers
- CLI bridge/release structured output and idempotency input
- MCP bridge mutation authority wrapping and settlement validation
- e2e harness driver-neutral receipt normalization and live parity scenario

Proof surface:
- `docs/validation/authoritative-live-settlement-driver-parity-slice.md`
- focused adapter contracts, cross-driver fixture matrix, and ignored-live test

Exact paths are finalized during RED after #7163 lands.

## Error Semantics

All drivers translate the same service error code without downgrading it to a
successful local state:
- missing authority remains `SERVICE_AUTHORITY_MISSING`
- authority mismatch remains the applicable bridge/settlement mismatch code
- malformed or invalid receipt digest remains a hard structured failure
- replay remains a hard idempotency/receipt-reuse failure

Adapters may add surface-specific context but must preserve the original code,
message, resource correlation, and cause. No silent fallback to legacy state.

## Test Plan

Red:
- Driver-neutral required-field and digest contract.
- Per-driver missing/partial/tampered/reordered/replayed fixture matrix.
- Cross-driver normalized-observation equality test.
- Real-entrypoint integration contract and documentation contract.

Green:
- Add the minimum shared fields to each adapter.
- Reuse one validator/normalizer where ownership boundaries permit.
- Wire real SDK, CLI, and MCP entrypoints into the parity scenario.

Refactor:
- Delete duplicated per-driver receipt parsing.
- Keep transport concerns separate from shared authority validation.
- Verify file/function size, naming, errors, and idempotency.

Integration:
- Execute all three real entrypoints with one operation identity.
- Verify identical receipt digest, signature, network, commitment, and one
  transfer across retry/restart.
- Re-run existing S-05, S-13, and Pi/MCP authority contracts.
