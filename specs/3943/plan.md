# Issue #3943 Plan

- Issue: #3943
- Status: In Progress
- Spec: `specs/3943/spec.md`

## Implementation Approach
1. Add deterministic remediation marker lines to the panic-policy checker section in `docs/ci/strategy.md`.
2. Add a dedicated docs-contract test in `crates/kamn-core/tests/ci_strategy_docs.rs` for panic-policy marker/remediation parity.
3. Run targeted and full `ci_strategy_docs` tests.

## Affected Modules
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations
- Risk: docs marker naming drift across future edits.
  - Mitigation: fail-closed docs-contract test with explicit marker assertions.
- Risk: remediation text becomes too free-form to assert.
  - Mitigation: enforce deterministic remediation marker keys/values.

## Contracts and Interfaces
- Required docs markers include:
  - checker command markers
  - panic-policy reason taxonomy markers
  - remediation marker/version entries

## Verification Strategy
- RED: add docs-contract assertions before docs markers are present.
- GREEN: add required docs markers/remediation entries.
- REGRESSION: run full `ci_strategy_docs` suite.
