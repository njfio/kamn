# Issue #4091 Plan

- Issue: #4091
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Approach
1. Add RED checker contract tests that reference a new quota checker API and taxonomy markers.
2. Implement `kamn-core` quota checker logic with fail-closed decisions and deterministic reason taxonomy helpers.
3. Add CI strategy docs marker section and docs-contract assertions.
4. Run targeted test/lint gates and set spec status to `Implemented`.

## Affected Files
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/src/quota_policy.rs` (new)
- `crates/kamn-core/tests/quota_policy_checker_contract.rs` (new)
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `docs/ci/strategy.md`
- `specs/4091/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: checker taxonomy drifts from fixture taxonomy.
  - Mitigation: explicit tests asserting taxonomy version and reason CSV parity.
- Risk: adding a new public module increases API surface unexpectedly.
  - Mitigation: keep module scope minimal and documented with deterministic contracts only.

## Interface Contract
- `quota_policy_reason_taxonomy_version() -> &'static str`
- `quota_policy_reason_codes_csv() -> &'static str`
- `evaluate_quota_policy(...) -> QuotaPolicyDecision`
