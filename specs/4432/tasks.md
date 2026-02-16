# Tasks: Issue #4432

Status: Completed
Issue: #4432

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Add subscription/packaging taxonomy RED assertions in:
  - `scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - `scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - `scripts/sdk/test_run_python_sdk_packaging_contract.sh`
  - `scripts/sdk/test_validate_python_sdk_packaging_live.sh`

T2 (GREEN, Implementation):
- Emit deterministic taxonomy markers in:
  - `scripts/sdk/run_rust_sdk_service_client_contract.sh`
  - `scripts/sdk/validate_rust_sdk_service_client_live.sh`
  - `scripts/sdk/run_python_sdk_packaging_contract.sh`
  - `scripts/sdk/validate_python_sdk_packaging_live.sh`

T3 (GREEN, Docs):
- Update:
  - `docs/sdk/rust-sdk.md`
  - `docs/sdk/python-sdk.md`
  - `docs/sdk/README.md`

T4 (Verify):
- Run:
  - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - `bash scripts/sdk/test_run_python_sdk_packaging_contract.sh`
  - `bash scripts/sdk/test_validate_python_sdk_packaging_live.sh`
  - `cargo test -p kamn-sdk --test service_api_client`
  - `python3 -m unittest tests.python.test_sdk`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-sdk -- -D warnings`

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
  - `cargo test -p kamn-sdk --test service_api_client`
    - Passed: `4 passed; 0 failed`
  - `python3 -m unittest tests.python.test_sdk`
    - Passed: `Ran 16 tests ... OK`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-sdk -- -D warnings`
    - Passed

- Regression summary:
  - Websocket subscription and packaging publish-readiness taxonomy markers are now deterministic in
    contract/live outputs.
  - SDK docs now pin subscription/packaging marker surfaces for drift detection.
