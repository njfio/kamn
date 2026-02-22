# Milestone R52 - E2E Live Runtime Integration Hardening

- Milestone: `R52 E2E Live Runtime Integration Hardening`
- Epic: #5611
- Completed issue(s): #5610, #5613, #5615, #5617, #5692, #5693, #5696, #5698, #5700, #5702, #5708, #5711, #5714, #5717, #5720, #5723, #5726, #5729, #5732, #5735, #5738, #5741, #5744, #5747, #5750, #5753
- Active issue(s): #5756
- Scope: harden external runtime execution readiness behavior with deterministic diagnostics while preserving phase-6 output contract stability, green-main quality gates, and branch-hygiene post-publication reconciliation evidence.

## Delivery Slices
1. External execution preflight executable diagnostics for `kolme_binary` and MCP `agent_binary`. (Completed)
2. External preflight rejection for non-file binary paths. (Completed)
3. External preflight absolute-path enforcement for binary paths. (Completed)
4. Integration config flag mapping correction (`agent_binary_required` vs `external_execution_enabled`). (Completed)
5. MCP JSON-RPC stdio protocol handling (`initialize`, `tools/list`, `tools/call`) with framed transport compatibility. (Completed)
6. Protocol-helper mutation hardening for MCP stdio handling. (Completed)
7. Opt-in live SDK-direct S-01 driver execution using `kamn-agent-lib` discovery signals. (Completed)
8. Opt-in live CLI-scripted S-01 driver execution using `kamn-cli health` command probe. (Completed)
9. Opt-in live MCP-agent S-01 driver execution using `kamn-mcp-server` command probe. (Completed)
10. Upgrade MCP-agent S-01 probe to framed JSON-RPC initialize + tools/call health flow. (Completed)
11. Align `kamn-cli` default/output contract with PRD JSON semantics. (Completed)
12. Restore R52 review marker and pre-merge workspace-test quality gates. (Completed)
13. Reconcile R52 branch-hygiene drift with merged-only cleanup and deterministic markers. (Completed)
14. Reconcile R52 post-publication quality-gate status markers with fail-closed docs-contract coverage. (Completed)
15. Execute R52 spec-volume remediation tranche-1 with deterministic 14-directory reduction evidence. (Completed)
16. Execute R52 spec-volume remediation tranche-2 with deterministic 14-directory reduction evidence. (Completed)
17. Execute R52 spec-volume remediation tranche-3 with deterministic 14-directory reduction evidence. (Completed)
18. Execute R52 spec-volume remediation tranche-4 with deterministic 14-directory reduction evidence. (Completed)
19. Execute R52 spec-volume remediation tranche-5 with deterministic 14-directory reduction evidence. (Completed)
20. Execute R52 spec-volume remediation tranche-6 with deterministic 14-directory reduction evidence. (Completed)
21. Execute R52 spec-volume remediation tranche-7 with deterministic 14-directory reduction evidence. (Completed)
22. Execute R52 spec-volume remediation tranche-8 with deterministic 14-directory reduction evidence. (Completed)
23. Execute R52 spec-volume remediation tranche-9 with deterministic 14-directory reduction evidence. (Completed)
24. Execute R52 spec-volume remediation tranche-10 with deterministic 14-directory reduction evidence. (Completed)
25. Execute R52 spec-volume remediation tranche-11 with deterministic 14-directory reduction evidence. (Completed)
26. Execute R52 spec-volume remediation tranche-12 with deterministic 14-directory reduction evidence. (Completed)
27. Reconcile R52 post-publication spec-volume guardrail status markers with fail-closed docs-contract coverage. (Completed)
28. Reconcile R52 post-publication priority-summary status markers with fail-closed docs-contract coverage. (Completed)
29. Reconcile R52 post-publication branch-hygiene status markers with fail-closed docs-contract coverage. (In Progress: #5756)
