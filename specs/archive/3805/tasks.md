# Issue #3805 Tasks

- Issue: `#3805`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add failing shell assertions for `observability_tls_negative_matrix_status` in runtime validation + policy + contract lane tests.
- T2 (Red): add failing observability endpoint TLS negative-matrix Rust tests (missing cert, invalid key, invalid mode, plain HTTP handshake rejection).
- T3 (Green): implement runtime lane marker propagation and required policy field enforcement for TLS negative matrix marker.
- T4 (Regression): add deterministic tamper checks for TLS negative matrix marker drift in policy suite.
- T5 (Docs): update runtime-network docs with observability TLS negative-path taxonomy and marker contracts.
- T6 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_missing_cert_file -- --exact --nocapture`
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_key_file -- --exact --nocapture`
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::regression_runtime_observability_endpoint_tls_mode_rejects_invalid_mode_value -- --exact --nocapture`
  - `cargo test -p kamn-node main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_rejects_plain_http_handshake -- --exact --nocapture`
  - `bash scripts/runtime/test_validate_runtime_observability_endpoint_live.sh`
  - `bash scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
  - `bash scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`

## Completion Evidence
- Observability TLS negative matrix fail-closed marker is emitted by runtime validation lane, enforced by policy + contract lane, and backed by deterministic Rust integration/regression tests.
