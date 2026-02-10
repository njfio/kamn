# Redaction and Tombstone Compliance Slice (Issues #150, #151)

This document describes the first implementation slice for PRD-aligned redaction and tombstone compliance workflows.

## Scope Delivered
- Added `RedactionComplianceEngine` in `kamn-core` for controlled redaction/tombstone workflows.
- Implemented quorum-based approval flow:
  - Requests remain pending until required approvals are collected.
  - Quorum automatically applies protection (redacted or tombstoned visibility).
- Implemented deterministic retrieval behavior:
  - `Available`
  - `Redacted { request_id }`
  - `Tombstoned { request_id }`
- Added immutable audit evidence stream per request:
  - `Requested`, `Approved`, `Rejected`, `Applied`
- Added target protection guard preventing silent restore/override for already protected targets.
- Integrated canonical state-key usage for target indexing.

## Classification/Redaction Compliance Contract Lane
Redaction and tombstone controls now share a deterministic compliance contract lane with classification evidence to keep audit posture reproducible.

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
- Shared compliance contract-lane module:
  - `scripts/compliance/classification_redaction_contract_lane_contract.py`

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
- the shell contract-lane wrapper delegates orchestration logic to `classification_redaction_contract_lane_contract.py` (`Regression: #1230`).

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test redaction_tombstones
cargo test -p kamn-core --test redaction_tombstones_docs
bash scripts/compliance/test_run_classification_redaction_lane.sh
bash scripts/compliance/test_check_classification_redaction_policy.sh
bash scripts/compliance/test_run_classification_redaction_contract_lane.sh
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add explicit requester/approver role policy checks and configurable authorization chains.
- Extend integration with storage and query APIs for operator-facing redaction history views.
