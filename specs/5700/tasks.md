# Tasks: #5700 Activate Opt-In Live MCP-Agent S-01 Driver Execution

- [x] T1 (Conformance/RED): add failing tests for MCP-agent live toggle and
  S-01 probe pass/fail mapping.
- [x] T2 (Implementation): implement configurable `McpAgentDriver` with env
  toggle and default `kamn-mcp-server` health probe runner.
- [x] T3 (Integration): wire run-contract MCP mode driver creation to env-aware
  `McpAgentDriver` constructor.
- [x] T4 (Regression): run `cargo test -p kamn-e2e-harness`.
- [x] T5 (Verify): run `cargo fmt --all --check` and
  `cargo clippy -p kamn-e2e-harness -- -D warnings`.
- [x] T6 (Closure): set spec status to Implemented and close issue with
  conformance + test telemetry.
