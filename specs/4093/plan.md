# Issue #4093 Plan

- Issue: #4093
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Approach
1. Add an explicit fairness docs-parity governance section to `docs/ci/strategy.md` with deterministic markers.
2. Add a new Rust contract test suite that:
   - reads checker taxonomy markers from `kamn_core::fairness_policy`,
   - verifies docs marker parity in `docs/ops/configuration.md` and `docs/ci/strategy.md`,
   - verifies each reason code has a remediation marker.
3. Add a docs-contract assertion in `ci_strategy_docs.rs` for marker presence and command references.
4. Run targeted tests and set spec status to `Implemented`.

## Affected Files
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/fairness_docs_parity_contract.rs` (new)
- `crates/kamn-core/tests/ci_strategy_docs.rs`
- `specs/4093/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: docs marker strings drift from checker reason taxonomy.
  - Mitigation: integration test compares checker CSV against docs marker CSV exactly.
- Risk: remediation mappings become partial/incomplete after reason-code changes.
  - Mitigation: regression test requires `fairness_docs_parity_remediation.<reason_code>=...` marker for every reason code.
- Risk: shell-surface expansion for governance checks.
  - Mitigation: keep implementation entirely in Rust tests/docs; no shell/workflow edits.

## Interface Contract
- Required docs strategy markers:
  - `fairness_docs_parity_reason_taxonomy_version=<version>`
  - `fairness_docs_parity_reason_codes_csv=<csv>`
  - `fairness_docs_parity_fixture_path=fixtures/runtime/starvation_fairness_fixture_matrix.txt`
  - `fairness_docs_parity_ops_doc_path=docs/ops/configuration.md`
  - `fairness_docs_parity_strategy_doc_path=docs/ci/strategy.md`
  - `fairness_docs_parity_remediation_map_version=v1`
- Required remediation marker key shape:
  - `fairness_docs_parity_remediation.<reason_code>=<operator action>`
