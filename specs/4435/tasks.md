# Tasks: Issue #4435

Status: Completed
Issue: #4435

## Ordered Tasks

T1 (RED):
- Add taxonomy/evidence marker assertions to:
  - `scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - `scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
- Add docs marker assertions for SDK HTTP helper taxonomy surfaces.

T2 (Verify RED):
- Run:
  - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
- Confirm deterministic failures before implementation changes.

## TDD Evidence

- RED command/output:
  - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - Failed with: `expected rust sdk service client contract request-error taxonomy marker`
  - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
    - Failed with: `expected rust sdk service client live http error taxonomy contract marker`

- GREEN command/output:
  - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - Passed: `rust sdk service client contract tests passed.`
  - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
    - Passed: `rust sdk service client live validation tests passed.`
