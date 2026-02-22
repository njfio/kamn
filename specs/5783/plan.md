# Plan: Issue #5783 — Extend Opt-In Live S-06 Proof-Verification Execution Across E2E Drivers

- Issue: #5783
- Spec: `specs/5783/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Add RED tests in each driver module proving live-enabled `S-06` dispatch and fail-closed behavior.
2. Implement scenario-aware live probe routing updates for `S-06` in:
   - `sdk_direct.rs`
   - `cli_scripted.rs`
   - `mcp_agent.rs`
3. Add dedicated live `S-06` probe helpers for SDK/CLI/MCP proof-verification surfaces with deterministic validation.
4. Preserve existing constructor compatibility (`with_probe`/`with_runner`) using additive per-scenario probe wiring.
5. Re-run targeted tests, crate checks, mutation gate, and workspace gate.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `specs/5783/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks / Mitigations
- Risk: live probe behavior diverges across SDK/CLI/MCP surfaces.
  - Mitigation: explicit per-scenario helpers and deterministic output checks for `verified/finality` markers.
- Risk: constructor API drift causes regression in existing tests.
  - Mitigation: keep existing constructors and add additive internals for new probe routing.

## Interfaces / Contracts
- Harness driver contract: `execute("S-06")` uses live probe only when live mode is enabled.
- No protocol/wire-format changes.

## ADR
- None required (no dependency/protocol/architecture decision changes).
