# Tasks: Issue #4431

Status: In Progress
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
  - Pending execution.

- GREEN command/output:
  - Pending implementation.

- Regression summary:
  - Pending verification.
