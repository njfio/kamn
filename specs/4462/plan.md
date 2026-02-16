# Plan: Issue #4462

Status: Completed
Issue: #4462

## Approach

1. Add RED tests for SLO threshold drift and SLO gate mismatch acceptance in go/no-go bundle tests.
2. Implement optional SLO policy gate inputs and deterministic gate payload building in
   `scripts/deploy/gonogo_evidence_contract.py`.
3. Wire checker-side deterministic SLO gate convergence validation.
4. Update release and observability docs, then bind with docs-contract tests.
5. Run scoped verification + hygiene gates.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/observability/schema.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/observability_schema_docs.rs`
- `specs/4462/*`

## Risks and Mitigations

- Risk: SLO gate reason drift causes unstable outputs.
  - Mitigation: exact-csv assertions in functional tests and deterministic code-path ordering.
- Risk: checker accepts tampered SLO gate payload.
  - Mitigation: strict deterministic rebuild + full-object equality convergence check.
- Risk: docs diverge from implementation markers.
  - Mitigation: docs-contract tests for release + observability docs.

## Interfaces / Contracts

- New go/no-go generate args:
  - `--slo-policy-report-file`
  - `--slo-policy-max-age-seconds`
- New go/no-go output markers:
  - `slo_policy_gate_final_decision`
  - `slo_policy_reason_taxonomy_version`
  - `slo_policy_reason_codes_csv`
  - `slo_policy_reason_codes_value`
- Deterministic checker mismatch message:
  - `slo policy gate convergence mismatch`

## ADR

Not required: no dependency or architecture changes.
