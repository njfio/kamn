# Issue 6269 - Stabilize signer emulator performance budget contract at boundary

## Objective
Stabilize the signer emulator performance contract test by removing boundary-condition flakiness while preserving a meaningful time budget assertion.

## Inputs/Outputs
- Inputs:
  - `KAMN_SIGNER_EMULATOR_CONTRACT_BUDGET_MS` (optional environment variable, integer milliseconds).
  - Runtime mode (`CI` set/unset) for default budget selection.
  - Test workload: 256 sign operations across secure mock and secure aws-kms emulator key identifiers.
- Outputs:
  - Deterministic pass/fail behavior for the performance contract at the budget boundary.
  - Clear failure message when the budget is exceeded.

## Boundaries/Non-goals
- In scope:
  - `crates/kamn-core/tests/signer_backend.rs` performance contract semantics.
  - Supporting test-only helper extraction if needed for readability/maintainability.
- Out of scope:
  - Re-architecting signer backend/router internals.
  - Changing unrelated test timing policies.
  - Removing or disabling the performance budget assertion.

## Failure modes
- FM1: Measured elapsed duration is above configured/default budget; test must fail with an explicit over-budget message.
- FM2: Boundary run equals configured/default budget and fails due to strict comparator semantics.
- FM3: Budget configuration is malformed and leads to ambiguous test behavior.

## Acceptance criteria (testable booleans)
- AC1: Running `cargo test -p kamn-core --test signer_backend performance_signer_emulator_contract_lane_stays_within_budget -- --exact --nocapture` passes on repeated local runs when elapsed duration is at or under budget.
- AC2: The performance contract still fails when elapsed duration is strictly above budget.
- AC3: `make test` passes from a clean branch after the fix.
- AC4: Comparator and budget parsing semantics are covered by explicit tests.

## Files to touch
- `crates/kamn-core/tests/signer_backend.rs`

## Error semantics
- Over-budget condition is a hard test failure with elapsed/budget context.
- Invalid explicitly configured budget value fails loudly in test context (panic with field/value context), not silently by fallback.
- Default budget selection remains explicit:
  - CI default: `600ms`
  - local default: `300ms`

## Test plan
- RED:
  - Add focused tests for budget comparator semantics and budget configuration parsing.
  - Ensure at least one new assertion fails against current strict `<` logic.
- GREEN:
  - Implement minimal comparator/parsing helper changes to satisfy tests.
  - Re-run targeted test file.
- REFACTOR:
  - Extract small helpers if needed to keep logic self-documenting.
- INTEGRATION:
  - Run `make test` to validate end-to-end lane health.

## Deviations
- Local default budget was finalized at `300ms` (not `250ms`) after repeated profiling showed local runtimes up to ~`260ms` and one cold-run outlier above `270ms`.
