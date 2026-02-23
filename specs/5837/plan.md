# Plan: Issue #5837 - Live S-15 Performance-Smoke Activation

- Issue: #5837
- Spec: `specs/5837/spec.md`
- Status: Implemented

## Approach
1. Add RED fail-closed tests for `execute("S-15")` across sdk-direct, cli-scripted, and mcp-agent.
2. Extend driver routing/state to include dedicated `S-15` probe functions in each driver.
3. Implement S-15 probe logic using existing send/query operation surfaces with deterministic latency sampling.
4. Add shared-per-driver helper validation for p50/p99/total budget contracts and unit tests for pass/fail branches.
5. Run targeted harness tests, docs-contract lanes, mutation-in-diff, and full quality gates.
6. Finalize lifecycle markers and milestone slice entry.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5837/spec.md`
- `specs/5837/plan.md`
- `specs/5837/tasks.md`

## Interfaces / Contracts
- Driver execution contract: `execute("S-15")` must fail closed on probe failure.
- Performance-smoke contract: measured latency samples must satisfy deterministic thresholds:
  - `max_total_ms`
  - `max_p50_ms`
  - `max_p99_ms`
- Existing operation-surface contract remains unchanged (reuse-only).

## Risks and Mitigations
- Risk: latency thresholds too strict could cause flakiness.
  - Mitigation: conservative defaults + env overrides + deterministic test fixtures.
- Risk: regressions in existing scenario routing while extending tuple wiring.
  - Mitigation: keep field ordering explicit and run full harness regression lane.

## ADR
- Not required: no new dependencies and no protocol/wire-format changes.
