# Issue #4092 Plan

- Issue: #4092
- Milestone: specs/milestones/r27-15-resource-quota-fairness-and-overload-resilience-governance/index.md

## Approach
1. Define a deterministic starvation fixture matrix with schema/taxonomy metadata and representative fairness scenarios.
2. Add a Rust fairness checker module with deterministic fail-closed reason codes.
3. Add checker contract tests that parse fixture metadata/cases and verify deterministic pass/fail outcomes.
4. Update `docs/ops/configuration.md` with fairness fixture markers and add matching docs-contract assertions.
5. Run targeted tests and update spec status to `Implemented`.

## Affected Files
- `fixtures/runtime/starvation_fairness_fixture_matrix.txt` (new)
- `crates/kamn-core/src/fairness_policy.rs` (new)
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/fairness_policy_checker_contract.rs` (new)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ops/configuration.md`
- `specs/4092/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: fixture schema drift from checker reason codes.
  - Mitigation: regression test enforces checker reason-code superset against fixture metadata.
- Risk: ambiguous fairness semantics across scope classes.
  - Mitigation: explicit allowed scope list plus deterministic reason markers for invalid inputs/starvation violations.
- Risk: shell surface creep while adding governance checks.
  - Mitigation: keep all implementation in Rust/tests/docs only; no script/workflow changes.

## Interface Contract
- Fixture metadata keys:
  - `fairness_fixture_matrix_schema_version`
  - `fairness_reason_taxonomy_version`
  - `fairness_reason_codes_csv`
- Fixture columns:
  - `case_id|scope|window_seconds|active_weighted_share|max_weighted_share_gap|expected_status|expected_reason_code`
- Checker outputs:
  - `Allow`
  - `Reject { reason: <deterministic reason> }`
