# Plan: Issue #4463

Status: In Progress
Issue: #4463

## Approach

1. Extend go/no-go evidence contract to support optional incident-readiness gate inputs.
2. Implement deterministic incident-readiness gate builder with schema/freshness/reason surface
   validation.
3. Add checker-side convergence validation for serialized incident-readiness gate payloads.
4. Add RED-first test coverage for stale/mismatch/tamper incident-readiness scenarios.
5. Update incident-readiness docs and docs-contract tests for drift protection.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `docs/ops/incident-readiness.md`
- `crates/kamn-core/tests/incident_readiness_docs.rs`

## Risks / Mitigations

- Risk: inconsistent incident report schema assumptions across scripts.
  - Mitigation: centralize source schema constants in go/no-go contract and validate deterministically.
- Risk: false-negative policy decisions from overly strict checks.
  - Mitigation: constrain required markers to explicitly documented contract surfaces only.
- Risk: docs drift after merge.
  - Mitigation: add dedicated docs contract tests and regression markers.

## Interfaces / Contracts

- New generator arguments:
  - `--incident-readiness-report-file`
  - `--incident-readiness-max-age-seconds`
- New payload field:
  - `incident_readiness_gate` object with schema/reason/artifacts/observed/contracts surfaces.
- New output markers:
  - `incident_readiness_gate_final_decision`
  - `incident_readiness_reason_taxonomy_version`
  - `incident_readiness_reason_codes_csv`

## ADR

No ADR required (no new dependency or protocol change).
