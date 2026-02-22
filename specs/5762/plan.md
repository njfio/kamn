# Plan: #5762 Add Real-Backend MCP Integration Contracts

## Approach
1. Add lifecycle artifacts and set milestone slice 31 in progress.
2. RED: author `crates/kamn-mcp-server/tests/real_backend_integration_contract.rs` with
   conformance tests for real-backend dispatch/stdio and invalid-request semantics; run targeted
   test expecting failures before implementation details are complete.
3. Implement fixture helpers and assertions needed for deterministic local service coverage.
4. Perform compensating archive cleanup for one archived issue-spec pair and update
   `specs/archive/index.md` to preserve `specs/` non-regression cap.
5. GREEN: run targeted integration test and cap-enforcement docs checks.
6. Verify with format and lint gates.

## Affected Modules / Files
- `crates/kamn-mcp-server/tests/real_backend_integration_contract.rs` (new)
- `specs/5762/spec.md`
- `specs/5762/plan.md`
- `specs/5762/tasks.md`
- `specs/archive/index.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: fixture server flakiness under timing variance.
  - Mitigation: deterministic non-blocking accept loop with bounded deadlines and fixed request budget.
- Risk: drift between dispatch and stdio expectations.
  - Mitigation: explicit per-path conformance assertions with stable marker strings.
- Risk: `specs/` cap breach due new lifecycle directory.
  - Mitigation: compensating archived pair cleanup and archive-policy checker.

## Interfaces / Contracts
New test contract file and conformance tests:
- `spec_c01_real_backend_dispatch_health_contract`
- `spec_c02_real_backend_dispatch_list_messages_contract`
- `spec_c03_real_backend_stdio_tools_call_contract`
- `spec_c04_real_backend_dispatch_invalid_request_contract`

## ADR
No ADR required (no architecture/protocol/dependency changes).
