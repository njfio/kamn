# Missing-Docs First Batch Graduation Report

schema_version: kamn.ci.kamn-core-missing-docs-graduation-batch-report.v1  
batch_id: first-three-modules-v1

## Representative Batch Modules

- bootstrap
- key_recovery
- kolme_runtime_commit

## Evidence Sources

- `fixtures/ci/kamn_core_missing_docs_graduated_modules.txt`
- `scripts/ci/check_kamn_core_missing_docs_policy.sh`
- `scripts/ci/missing_docs_throughput_report_contract.py`
- `scripts/ci/missing_docs_velocity_guard.py`

## Verification Commands

- `bash scripts/ci/test_missing_docs_graduation_batch_report_contract.sh`
- `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`

## Notes

- The allow-list fixture is intentionally empty; this report preserves a durable
  first-batch marker set for issue-driven traceability.
- Batch module names must remain present in
  `fixtures/ci/kamn_core_missing_docs_graduated_modules.txt`.

Regression: #2126
