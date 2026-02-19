# Plan: Issue #4461

Status: Completed
Issue: #4461

## Approach

1. Add RED coverage to go/no-go bundle tests for audit-integrity convergence markers, tamper
   rejection, and deterministic reason outputs.
2. Extend `scripts/deploy/gonogo_evidence_contract.py` with optional audit-integrity gate inputs
   and a deterministic gate payload builder.
3. Wire checker-side deterministic convergence validation for audit gate payloads.
4. Update release and ops docs contracts to pin audit marker references.
5. Run scoped verification commands and capture red/green evidence in tasks.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `specs/4461/*`

## Risks and Mitigations

- Risk: additional optional gate complicates decision logic.
  - Mitigation: keep audit gate optional, deterministic, and isolated like TLS gate.
- Risk: reason output drift can regress downstream parsers.
  - Mitigation: enforce exact reason marker/csv checks in functional tests.
- Risk: checker accepts tampered payload surface.
  - Mitigation: strict deterministic re-build + full object equality check.

## Interfaces / Contracts

- New go/no-go generate args:
  - `--audit-integrity-report-file`
  - `--audit-integrity-max-age-seconds`
- New go/no-go output markers:
  - `audit_integrity_gate_final_decision`
  - `audit_integrity_reason_taxonomy_version`
  - `audit_integrity_reason_codes_csv`
  - `audit_integrity_reason_codes_value`
- Deterministic checker mismatch message:
  - `audit integrity gate convergence mismatch`

## ADR

Not required: no dependency or architecture change.
