# Data Classification and Write-Tagging Slice (Issues #156, #157)

This document describes the first implementation slice for classification tiers and write-path tagging enforcement.

## Scope Delivered
- Added `DataClassificationEngine` to enforce write-path classification controls.
- Added versionable classification model:
  - `Public`
  - `Internal`
  - `Sensitive`
  - `Restricted`
- Added domain-level minimum classification policy:
  - `messages`, `tasks`, `escrows`, `reputation`
- Added required-tag enforcement by classification level.
- Enforced typed write-path failures for:
  - Missing required tags
  - Classification below domain minimum
  - Sensitive/restricted writes without tags
  - Invalid actor DIDs and malformed policy/tag definitions
- Added operator-facing status surface (`ClassificationStatus`) for deterministic control visibility.
- Integrated canonical write-key output through existing state key normalization.

## DSAR/Export/Erasure Legal-Hold Evidence Contract (Issue #746)
GDPR data-subject workflows require deterministic legal-hold precedence before export/erasure decisions can proceed.

- Evidence bundle generator:
  - `bash scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh --output-file /tmp/dsar-legal-hold.json --request-id dsar-erasure-001 --subject-did did:kamn:subject-001 --request-type ERASURE --legal-hold-active true --retention-expired true --evidence-complete true --approval-recorded true --tamper-check PASS --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/compliance/check_dsar_legal_hold_policy.sh --bundle-file /tmp/dsar-legal-hold.json`
- PR fast contract lane:
  - `bash scripts/compliance/run_dsar_legal_hold_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/compliance/run_dsar_legal_hold_deep_lane.sh --output-json dsar-legal-hold-report.json`
- Replay matrix runner:
  - `python3 scripts/compliance/run_dsar_legal_hold_matrix.py --fixture fixtures/compliance_dsar/legal_hold_precedence_cases.json --output-json dsar-legal-hold-report.json`
- Regression policy:
  - legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`).

## Classification/Redaction Compliance Contract Lane
Classification and redaction controls are validated together through a deterministic compliance lane that fails closed on schema or docs drift.

- Compliance lane command:
  - `bash scripts/compliance/run_classification_redaction_lane.sh --output-file /tmp/classification-redaction-report.json`
- Shared compliance lane module:
  - `scripts/compliance/classification_redaction_lane_contract.py`
- Compliance policy checker:
  - `bash scripts/compliance/check_classification_redaction_policy.sh --report-file /tmp/classification-redaction-report.json`
- Shared compliance policy module:
  - `scripts/compliance/classification_redaction_policy_contract.py`
- Compliance contract lane:
  - `bash scripts/compliance/run_classification_redaction_contract_lane.sh --output-file /tmp/classification-redaction-contract-report.json`

Runtime budget controls:

- `KAMN_CLASSIFICATION_REDACTION_MAX_SECONDS`
- `KAMN_CLASSIFICATION_REDACTION_CONTRACT_MAX_SECONDS`
- `KAMN_CLASSIFICATION_REDACTION_SKIP_COMMANDS`

Required schema/reason markers:

- `kamn.compliance.classification-redaction-report.v1`
- `classification_redaction_reason_codes:GO:v1`
- `classification_redaction_reason_codes:NO-GO:v1`

Regression policy:

- classification/redaction contract drift must fail closed (`Regression: #914`).
- the shell lane wrapper delegates orchestration logic to `classification_redaction_lane_contract.py` (`Regression: #1226`).
- the shell policy wrapper delegates validation logic to `classification_redaction_policy_contract.py` (`Regression: #1222`).

## Local Validation
Run from repository root:

```bash
bash scripts/compliance/test_generate_dsar_legal_hold_evidence_bundle.sh
bash scripts/compliance/test_run_dsar_legal_hold_contract_lane.sh
bash scripts/compliance/test_run_dsar_legal_hold_matrix.sh
bash scripts/compliance/test_run_dsar_legal_hold_deep_lane.sh
bash scripts/compliance/test_run_classification_redaction_lane.sh
bash scripts/compliance/test_check_classification_redaction_policy.sh
bash scripts/compliance/test_run_classification_redaction_contract_lane.sh
cargo test -p kamn-core --test data_classification_tagging
cargo test -p kamn-core --test data_classification_tagging_docs
cargo test -p kamn-core --test redaction_tombstones_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add classification inheritance for nested write contexts.
- Add policy-version tracking and migration rules for classification updates.
