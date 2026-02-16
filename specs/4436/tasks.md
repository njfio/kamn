# Tasks: Issue #4436

Status: Completed
Issue: #4436

## Ordered Tasks

T1 (Input from #4435 RED):
- Consume deterministic RED failures for missing taxonomy/evidence markers.

T2 (GREEN, Implementation):
- Implement taxonomy/evidence marker outputs in:
  - `scripts/sdk/run_rust_sdk_service_client_contract.sh`
  - `scripts/sdk/validate_rust_sdk_service_client_live.sh`

T3 (GREEN, Python Regression):
- Add deterministic legacy adapter reason normalization regressions in:
  - `tests/python/test_sdk.py`

T4 (GREEN, Docs):
- Update:
  - `docs/sdk/rust-sdk.md`
  - `docs/sdk/README.md`

T5 (Verify):
- Run:
  - `bash scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - `bash scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
  - `cargo test -p kamn-sdk --test service_api_client`
  - `python3 -m unittest tests.python.test_sdk`
  - `cargo fmt --check`
  - `cargo clippy -p kamn-sdk -- -D warnings`

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
  - `python3 -m unittest tests.python.test_sdk`
    - Passed: `Ran 16 tests ... OK`
