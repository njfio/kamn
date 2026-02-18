# Issue #4316 Tasks

- Issue: `#4316`
- Status: `Completed`

## Ordered Tasks
- T1 (Red): add lifecycle-limiter reason projection conformance tests in `service_api_endpoint_tests` and capture failing evidence.
- T2 (Green): implement shared lifecycle-limiter rejection projection mapping and apply it in middleware rejection paths.
- T3 (Docs): add lifecycle rejection taxonomy section to `docs/service/api-contract.md` and guard with docs test.
- T4 (Verify): run
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo test -p kamn-node lifecycle_rejection_projection -- --nocapture`
  - `cargo test -p kamn-core --test service_api_lifecycle_contract_docs`

## Completion Evidence
- Projection helper output is deterministic and used by limiter rejection paths.
- Live concurrency-limiter rejection maps to stable projection class/status/outcome.
- Docs taxonomy markers are enforced by tests.
- Verification commands passed:
  - `cargo fmt --check`
  - `cargo clippy -p kamn-node -- -D warnings`
  - `cargo test -p kamn-node unit_service_api_endpoint_lifecycle_rejection_projection_is_deterministic -- --exact` (RED first failed on missing imports, then passed after implementation)
  - `cargo test -p kamn-node lifecycle_rejection_projection -- --nocapture`
  - `cargo test -p kamn-node lifecycle_projection_ -- --nocapture`
  - `cargo test -p kamn-core --test service_api_lifecycle_contract_docs`
  - `cargo mutants --in-diff` (tool unavailable in environment)
