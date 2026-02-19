# Tasks: Issue #4431

Status: Completed
Issue: #4431

## Ordered Tasks

T1 (RED, Functional/Conformance):
- Add RED assertions for taxonomy/evidence marker surfaces in:
  - `scripts/sdk/test_run_rust_sdk_service_client_contract.sh`
  - `scripts/sdk/test_validate_rust_sdk_service_client_live.sh`
- Add docs marker assertions for `docs/sdk/rust-sdk.md` and `docs/sdk/README.md`.

T2 (GREEN, Implementation):
- Emit deterministic request-error taxonomy markers in:
  - `scripts/sdk/run_rust_sdk_service_client_contract.sh`
  - `scripts/sdk/validate_rust_sdk_service_client_live.sh`
- Ensure corresponding JSON report fields are present and stable.

T3 (GREEN, Python Regression):
- Add deterministic legacy adapter reason normalization regression checks to:
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
  - `cargo test -p kamn-sdk --test service_api_client`
    - Passed: `4 passed; 0 failed`
  - `python3 -m unittest tests.python.test_sdk`
    - Passed: `Ran 16 tests ... OK`
  - `cargo fmt --check`
    - Passed
  - `cargo clippy -p kamn-sdk -- -D warnings`
    - Passed

- Regression summary:
  - Rust SDK HTTP contract/live outputs now emit deterministic request-error taxonomy markers.
  - Python legacy adapter reason normalization is regression-pinned for deterministic fallbacks.
  - SDK docs now pin taxonomy/evidence marker surfaces in `docs/sdk/rust-sdk.md` and
    `docs/sdk/README.md`.
