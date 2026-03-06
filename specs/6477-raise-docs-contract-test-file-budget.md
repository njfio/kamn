# Spec: Issue 6477 - Raise docs contract test file budget

## Objective
Update the docs-contract test-file budget so Fast Gate matches the current
repository inventory of `*_docs.rs` tests and stops failing on intentional
existing coverage.

## Inputs/Outputs
- Inputs:
  - `.ci/docs-contract-test-file-budget.env`
  - `.github/workflows/ci-fast-gate.yml`
- Outputs:
  - Updated docs-contract test file budget ceiling.
  - Regression evidence that the exact Fast Gate count check fails before the
    change and passes after it.

## Boundaries/Non-goals
- No workflow logic changes.
- No reduction of the current docs-contract test inventory.
- No changes to non-docs test-file budgets.

## Failure modes
- Fast Gate keeps failing because the budget ceiling remains below the actual
  docs-contract file count.
- The budget is raised to a value that does not match the current inventory.
- The issue is resolved by weakening unrelated CI checks instead of correcting
  the budget source of truth.

## Acceptance criteria (testable booleans)
- [ ] AC-1: `.ci/docs-contract-test-file-budget.env` sets
      `DOCS_CONTRACT_TEST_FILE_MAX=69`.
- [ ] AC-2: The exact Fast Gate docs-contract count check fails before the
      budget update with `69 > 65`.
- [ ] AC-3: The same count check passes after the budget update with the current
      repository inventory.
- [ ] AC-4: The spec records the budget rationale and validation evidence.

## Files to touch
- `specs/6477-raise-docs-contract-test-file-budget.md`
- `.ci/docs-contract-test-file-budget.env`

## Error semantics
- Documentation/configuration change only; no runtime error behavior changes.
- The Fast Gate count check must continue to fail loudly if inventory exceeds
  the configured ceiling.

## Test plan
- Red:
  - Run the exact docs-contract count check from `ci-fast-gate.yml` and confirm
    it fails against the current budget.
- Green:
  - Raise the budget to the current inventory count.
- Refactor:
  - Keep the change scoped to the budget source of truth only.
- Integration:
  - Re-run the exact Fast Gate count check and confirm it passes.

## Phase 6 integration evidence
- Pending.

## Deviations
- None.
