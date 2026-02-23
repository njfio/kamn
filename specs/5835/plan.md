# Plan: Issue #5835 - Live S-14 Batch-Merkle Activation

- Issue: #5835
- Spec: `specs/5835/spec.md`
- Status: Implemented

## Approach
1. Add RED fail-closed tests for `execute("S-14")` across sdk-direct, cli-scripted, and mcp-agent drivers.
2. Wire live-route mapping for `S-14` in all three drivers.
3. Implement deterministic `S-14` probe runners using existing operation surfaces:
   - send two messages,
   - query both messages,
   - verify both proofs against one shared deterministic batch-root marker.
4. Add focused probe-level unit guards (missing binary / invalid endpoint) matching S-12/S-13 style.
5. Run targeted package tests, then harness + docs-contract regression lanes.
6. Finalize lifecycle markers and milestone slice entry.

## Affected Modules
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5835/spec.md`
- `specs/5835/plan.md`
- `specs/5835/tasks.md`

## Interfaces / Contracts
- Driver execution contract: `execute("S-14")` must fail closed when live S-14 probe fails.
- Probe continuity contract: both proof verification calls must return verified-final responses for both batched message IDs under one shared root marker.
- Existing CLI/MCP/API surface contract remains unchanged (reuse-only).

## Risks and Mitigations
- Risk: accidentally regressing existing `S-13` and earlier routing maps while extending tuple wiring.
  - Mitigation: preserve field ordering and run focused fail-closed tests for S-12/S-13/S-14 plus full harness lane.
- Risk: brittle response parsing for verify outputs across text/json projections.
  - Mitigation: mirror existing parser/helpers and validate required fields explicitly.

## ADR
- Not required: no new dependency and no wire/protocol changes.
