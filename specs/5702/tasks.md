# Tasks: #5702 Upgrade MCP-Agent Live S-01 Probe to Framed JSON-RPC Flow

- [x] T1 (Conformance/RED): add failing helper tests for framed MCP request
  generation and framed response parsing/validation.
- [x] T2 (Implementation): upgrade MCP live probe to framed JSON-RPC initialize
  + tools/call health sequence.
- [x] T3 (Regression): run `cargo test -p kamn-e2e-harness`.
- [x] T4 (Verify): run `cargo fmt --all --check` and
  `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T5 (Mutation): run in-diff mutation checks for harness slice.
- [x] T6 (Closure): set spec status to Implemented and close issue with
  telemetry.
