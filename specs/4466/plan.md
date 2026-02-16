# Plan: Issue #4466

Status: Completed
Issue: #4466

## Approach

1. Add deterministic audit-integrity taxonomy constants in go/no-go contract code.
2. Normalize audit gate reason outputs for both marker and JSON payload surfaces.
3. Update release go/no-go checklist with audit taxonomy requirements and regression policy.
4. Verify with deploy contract tests and docs-contract tests.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract.py`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `specs/4466/*`

## Risks and Mitigations

- Risk: reason-code ordering drift.
  - Mitigation: sort/de-duplicate reason codes and assert exact csv output in tests.
- Risk: docs/reference drift from implementation.
  - Mitigation: add docs-contract test assertions for exact marker lines.

## Interfaces / Contracts

- Audit-integrity markers:
  - `audit_integrity_reason_taxonomy_version`
  - `audit_integrity_reason_codes_csv`
  - `audit_integrity_reason_codes_value`
  - `audit_integrity_gate_final_decision`
