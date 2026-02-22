# Tasks: Issue #5799 — Align SDK/Agent-Lib Protected-Route Auth with Service Scope Policy

- Issue: #5799
- Spec: `specs/5799/spec.md`
- Plan: `specs/5799/plan.md`
- Status: Completed
- Last Updated: 2026-02-22

## Ordered Tasks
1. T1 (RED/Conformance): add failing tests for protected-route scope header emission and chain-context override behavior. ✅
2. T2 (GREEN): implement optional scope support in SDK request auth and HTTP header emission. ✅
3. T3 (GREEN): implement chain-context override + scope-aware auth building in agent-lib. ✅
4. T4 (GREEN): update `KamnAgentHandle` protected-route methods to pass policy-correct scopes. ✅
5. T5 (Regression): run scoped crate tests for sdk/agent-lib/cli/mcp/e2e harness interfaces. ✅
6. T6 (GREEN): update S-04 live probe paths in sdk/cli/mcp drivers to avoid replay/anti-spam rejection. ✅
7. T7 (Functional): rerun S-01/S-04/S-06 live matrix against local `kamn-node` and update evidence artifact. ✅
8. T8 (Lifecycle): finalize spec/tasks statuses, issue log, and PR evidence. ✅

## AC/Tier Mapping
- AC-1: T1, T2, T4 (Integration)
- AC-2: T1, T3 (Unit)
- AC-3: T6, T7 (Functional/Regression)
- AC-4: T5 (Regression)

## Verification Snapshot
- `cargo test -p kamn-sdk --test service_api_client` ✅
- `cargo test -p kamn-agent-lib` ✅
- `cargo test -p kamn-cli` ✅
- `cargo test -p kamn-mcp-server` ✅
- `cargo test -p kamn-e2e-harness` ✅
- Live matrix output artifacts:
  - `.tmp/5799-live/sdk-direct.json` (`S-01/S-04/S-06=PASS`)
  - `.tmp/5799-live/cli-scripted.json` (`S-01/S-04/S-06=PASS`)
  - `.tmp/5799-live/mcp-tau.json` (`S-01/S-04/S-06=PASS`)
