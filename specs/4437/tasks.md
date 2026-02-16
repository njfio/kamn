# Tasks: Issue #4437

Status: Completed
Issue: #4437

## Ordered Tasks

T1 (RED):
- Add RED taxonomy drift assertions to:
  - `scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - `scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - `scripts/sdk/test_run_python_sdk_packaging_contract.sh`
  - `scripts/sdk/test_validate_python_sdk_packaging_live.sh`

T2 (Verify RED):
- Run:
  - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
  - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`

## TDD Evidence

- RED command/output:
  - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - Failed with: `expected rust sdk service client contract subscription taxonomy marker`
  - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
    - Failed with: `expected rust sdk service client live subscription contract marker`
  - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
    - Failed with: `expected python sdk packaging publish-readiness taxonomy marker`
  - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
    - Failed with: `expected python sdk packaging live publish-readiness taxonomy status marker`

- GREEN command/output:
  - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
    - Passed: `rust sdk service client contract tests passed.`
  - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
    - Passed: `rust sdk service client live validation tests passed.`
  - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
    - Passed: `python sdk packaging contract runner tests passed.`
  - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
    - Passed: `python sdk packaging live validation tests passed.`
