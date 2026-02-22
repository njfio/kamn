# Plan: Issue #5781 — Extend Opt-In Live S-04 Task Lifecycle Execution Across E2E Drivers

- Issue: #5781
- Spec: `specs/5781/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Add RED tests in each driver module proving `S-04` live dispatch and fail-closed behavior.
2. Implement scenario-aware live probe dispatch in:
   - `sdk_direct.rs`
   - `cli_scripted.rs`
   - `mcp_agent.rs`
3. Preserve current public constructors with compatibility wrappers, adding explicit multi-probe constructors where needed.
4. Re-run targeted harness tests, then formatting/lint and workspace verification.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `specs/5781/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks / Mitigations
- Risk: live probe fragility in real environments.
  - Mitigation: explicit opt-in gates remain; probes return deterministic error text.
- Risk: constructor API drift breaking existing tests.
  - Mitigation: keep existing constructors and add additive constructors for multi-probe injection.

## Interfaces / Contracts
- Harness driver contract: `execute("S-04")` uses live probe only when driver live mode is enabled.
- No service protocol changes.

## ADR
- None required (no architecture/protocol/dependency changes).
