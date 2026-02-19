# Plan: Issue #4468

Status: Completed
Issue: #4468

## Approach

1. Add deterministic SLO gate taxonomy constants and normalized outputs in go/no-go contract code.
2. Validate SLO gate convergence deterministically in checker path.
3. Update release/observability docs and docs-contract assertions for SLO taxonomy markers.
4. Verify via deploy contract tests and docs tests.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/observability/schema.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/observability_schema_docs.rs`
- `specs/4468/*`

## Risks and Mitigations

- Risk: reason-code ordering drift.
  - Mitigation: sorted/de-duplicated reason list and exact csv assertions.
- Risk: docs parity drift.
  - Mitigation: dedicated docs-contract tests for SLO taxonomy section markers.

## Interfaces / Contracts

- SLO gate markers:
  - `slo_policy_reason_taxonomy_version`
  - `slo_policy_reason_codes_csv`
  - `slo_policy_reason_codes_value`
  - `slo_policy_gate_final_decision`
