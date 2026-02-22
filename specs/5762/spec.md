# Spec: #5762 Add Real-Backend MCP Integration Contracts

- Issue: #5762
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Reviewed
- Priority: P1

## Problem Statement
R52 delivered functional MCP dispatch capability, but current `kamn-mcp-server` coverage is mostly
synthetic backend testing. We need fail-closed integration tests that exercise the production
`impl McpToolBackend for KamnAgentHandle` path against deterministic local HTTP fixtures so the
portable-agent runtime path is validated end-to-end.

## Scope
### In Scope
- Add RED-first integration tests in `crates/kamn-mcp-server/tests` that instantiate a real
  `KamnAgentHandle` and dispatch through `kamn-mcp-server` APIs.
- Validate at least one direct dispatch path (`dispatch_tool_request_json`) and one framed JSON-RPC
  stdio path (`process_stdio_input`) using deterministic loopback service responses.
- Keep test fixtures local-only and deterministic (no external services).
- Update milestone index and issue lifecycle artifacts.
- Perform compensating single archived issue-spec pair cleanup to preserve the top-level `specs/`
  non-regression cap (`<= 693`) after adding `specs/5762`.

### Out of Scope
- CI workflow topology changes.
- Production runtime behavior changes in `kamn-agent-lib` or `kamn-mcp-server`.
- Protocol schema redesign.

## Acceptance Criteria
### AC-1 Real-backend dispatch integration coverage
Given a deterministic local service fixture,
When `dispatch_tool_request_json` is called with a real `KamnAgentHandle` backend,
Then health and list-messages tool calls succeed with structured payload markers.

### AC-2 Real-backend framed stdio integration coverage
Given framed JSON-RPC `tools/call` requests,
When `process_stdio_input` executes with a real `KamnAgentHandle` backend,
Then framed responses include expected JSON-RPC and result payload markers.

### AC-3 Deterministic and fail-closed tests
Given malformed inputs or missing fields,
When the integration contract lane runs,
Then failures remain deterministic and express structured invalid-request semantics.

### AC-4 Non-regression cap preservation
Given one new lifecycle spec directory is added,
When implementation completes,
Then top-level `specs/` directory count remains `<= 693` via compensating archive cleanup.

## Conformance Cases
- C-01 (AC-1): `spec_c01_real_backend_dispatch_health_contract` passes.
- C-02 (AC-1): `spec_c02_real_backend_dispatch_list_messages_contract` passes.
- C-03 (AC-2): `spec_c03_real_backend_stdio_tools_call_contract` passes.
- C-04 (AC-3): `spec_c04_real_backend_dispatch_invalid_request_contract` passes.
- C-05 (AC-4): `cargo test -p kamn-core --test review_r50_spec_volume_remediation_docs_contract`.
- C-06 (AC-4): `bash scripts/ci/check_spec_archive_policy.sh --repo-root . --output-json <path>`.
- C-07 (AC-1..AC-4): `cargo test -p kamn-mcp-server --test real_backend_integration_contract`.
- C-08 (AC-1..AC-4): `cargo fmt --all --check`.
- C-09 (AC-1..AC-4): `cargo clippy -p kamn-mcp-server --tests -- -D warnings`.

## Success Metrics / Observable Signals
- New real-backend integration tests run green and validate both dispatch and stdio paths.
- Structured invalid-request semantics remain stable in integration coverage.
- Top-level `specs/` directory cap remains compliant (`<= 693`).
