# Issue #7152: Migrate Pi Actor Contract To Authority Taxonomy

## Objective

Migrate the Pi transaction actor contract from pre-#7135 fixture and error assumptions
to transaction-bound v2 authority and exact verifier categories.

## Inputs And Outputs

- Input: bound v2 actor artifacts with optional process, identity, projection, privacy,
  shared-fact, authorization, operation, or type drift.
- Output: success for coherent authority and exact fail-closed categories for drift.

## Boundaries And Non-Goals

- Do not change production verifier behavior or ordering.
- Do not accept unbound, v1, or client-local authority.
- Do not change settlement, durable receipt-chain, Pi transport, or governance code.

## Failure Modes

- The success case uses generic unbound v2 evidence.
- Process reuse is conflated with service-authority drift.
- Identity, projection, privacy, shared-fact, or authorization drift uses obsolete
  pre-authority categories.
- Missing or type-confused operations are accepted.

## Acceptance Criteria

- [x] The valid path uses transaction-bound v2 actor evidence.
- [x] Duplicate process returns `PI_TRANSPORT_PROVENANCE_INVALID`.
- [x] Identity, projection, privacy, shared-fact, and authorization drift return
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- [x] Missing operation and type confusion remain fail-closed.
- [x] The complete four-case target passes.
- [x] Formatting and strict Clippy pass.

## Files To Touch

- `specs/7152-pi-actor-authority-taxonomy.md`
- `crates/kamn-e2e-harness/tests/mvp_demo_pi_transaction_actor_contract.rs`

## Error Semantics

- Actor service authority or projection disagreement:
  `PI_SERVICE_AUTHORITY_MISMATCH`.
- Actor process or transport provenance disagreement:
  `PI_TRANSPORT_PROVENANCE_INVALID`.
- Missing, malformed, or type-confused actor operations fail with the owning `PI_*`
  boundary and are never accepted.

## Test Plan

### RED

- Name the existing fixture and category assumptions and reproduce the three failures.

### GREEN

- Use bound v2 fixtures and exact service-authority/transport categories.

### REFACTOR

- Centralize bound fixture verification helpers within the target.

### INTEGRATION

- Run the complete target, formatting, strict Clippy, and adjacent authority contracts.

## Verification Evidence

- `cargo fmt --all -- --check`
- `cargo test -p kamn-e2e-harness --test mvp_demo_pi_transaction_actor_contract
  --test mvp_demo_pi_service_authority_contract`
  - Result: 6 passed, 0 failed.
- `cargo clippy -p kamn-e2e-harness --test mvp_demo_pi_transaction_actor_contract
  --test mvp_demo_pi_service_authority_contract -- -D warnings`
- A broader runtime receipt-chain target run exposed a separate unbound success fixture;
  that target is outside this issue and remains fail-closed.
