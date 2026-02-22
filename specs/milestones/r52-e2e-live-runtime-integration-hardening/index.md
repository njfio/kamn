# Milestone R52 - E2E Live Runtime Integration Hardening

- Milestone: `R52 E2E Live Runtime Integration Hardening`
- Epic: #5611
- Completed issue(s): #5610, #5613, #5615, #5617, #5692, #5693, #5696, #5698
- Active issue(s): #5700
- Scope: harden external runtime execution readiness behavior with deterministic diagnostics while preserving phase-6 output contract stability.

## Delivery Slices
1. External execution preflight executable diagnostics for `kolme_binary` and MCP `agent_binary`. (Completed)
2. External preflight rejection for non-file binary paths. (Completed)
3. External preflight absolute-path enforcement for binary paths. (Completed)
4. Integration config flag mapping correction (`agent_binary_required` vs `external_execution_enabled`). (Completed)
5. MCP JSON-RPC stdio protocol handling (`initialize`, `tools/list`, `tools/call`) with framed transport compatibility. (Completed)
6. Protocol-helper mutation hardening for MCP stdio handling. (Completed)
7. Opt-in live SDK-direct S-01 driver execution using `kamn-agent-lib` discovery signals. (Completed)
8. Opt-in live CLI-scripted S-01 driver execution using `kamn-cli health` command probe. (Completed)
9. Opt-in live MCP-agent S-01 driver execution using `kamn-mcp-server` command probe. (In Progress)
