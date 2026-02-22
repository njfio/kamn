# Tasks: #5762 Add Real-Backend MCP Integration Contracts

- [x] T1 (Conformance/RED): add real-backend integration tests in `real_backend_integration_contract.rs` and run target lane to capture initial failing evidence.
- [x] T2 (Implementation): implement deterministic fixture helpers and test assertions for dispatch and framed stdio paths.
- [x] T3 (Implementation): add invalid-request fail-closed integration assertion.
- [x] T4 (Implementation): perform compensating single archived issue-spec pair cleanup and update `specs/archive/index.md`.
- [x] T5 (Conformance/GREEN): run `cargo test -p kamn-mcp-server --test real_backend_integration_contract`.
- [x] T6 (Conformance/GREEN): run `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract` and archive-policy checker.
- [x] T7 (Verify): run `cargo fmt --all --check` and `cargo clippy -p kamn-mcp-server --tests -- -D warnings`.
- [x] T8 (Closure): set spec status to Implemented, update milestone index, close issue.
