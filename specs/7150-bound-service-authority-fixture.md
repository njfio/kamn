# Issue #7150: Use Transaction-Bound Service-Authority Fixture

## Objective

Align the valid service-authority success contract with the transaction-bound v2 actor
fixture required by the independent verifier.

## Inputs And Outputs

- Input: three role-scoped v2 actor artifacts bound to the canonical transaction facts.
- Output: a verified service-authority summary containing receipt-chain and public
  commitments.

## Boundaries And Non-Goals

- Do not change production verifier behavior.
- Do not accept generic unbound v2 or client-local v1 authority.
- Do not change settlement, durable receipt-chain, Pi transport, or governance behavior.

## Failure Modes

- The valid-path test uses an unbound fixture and fails with
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- Transaction binding is weakened to make the stale fixture pass.
- The v1/client-local negative case becomes accepted.

## Acceptance Criteria

- [ ] The valid service-authority contract uses the transaction-bound v2 fixture.
- [ ] The v1/client-local authority case remains rejected with
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- [ ] The complete `mvp_demo_pi_service_authority_contract` target passes.
- [ ] Formatting and strict Clippy pass.

## Files To Touch

- `specs/7150-bound-service-authority-fixture.md`
- `crates/kamn-e2e-harness/tests/mvp_demo_pi_service_authority_contract.rs`

## Error Semantics

- Bound v2 authority verifies successfully.
- Unbound, v1, or client-local authority returns `PI_SERVICE_AUTHORITY_MISMATCH`.

## Test Plan

### RED

- Reproduce the valid-path failure with the generic v2 fixture.

### GREEN

- Use the transaction-bound v2 fixture for the success case.

### REFACTOR

- Name the fixture setup helper for the valid independent-authority boundary.

### INTEGRATION

- Run the full contract target, formatting, and strict Clippy.
