# Issue #3930 Plan

- Issue: #3930
- Status: Reviewed

## Approach
1. Validate current peer-lifecycle proptest coverage against ACs and identify missing explicit contract markers.
2. Add red-first docs-contract assertions for peer lifecycle anti-churn and bounded-envelope markers in `runtime_network_docs.rs`.
3. Add explicit budget-envelope unit assertion in `peer_lifecycle_proptest_invariants.rs`.
4. Update `docs/foundation/runtime-network.md` with required marker text to satisfy docs contracts.
5. Run target suites + fmt/clippy + shell guardrails and prepare closure artifacts.

## Risks and Mitigations
- Risk level: medium
- Risks:
  - Property lane semantics can drift if anti-churn expectations are not explicitly pinned.
  - Documentation can diverge from behavior unless contract tests pin required markers.
- Mitigations:
  - Add fail-closed assertions in docs-contract tests.
  - Keep implementation scoped to tests/docs with no production logic changes.

## Interface Contract
- Test and documentation contract hardening only.
- No runtime API or wire-format changes.

## ADR
- Not required (no architectural decision change).
