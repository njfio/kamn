# Plan: Issue #5797 — Execute Live Harness S-01/S-04/S-06 Across SDK/CLI/MCP

- Issue: #5797
- Spec: `specs/5797/spec.md`
- Status: Reviewed
- Last Updated: 2026-02-22

## Implementation Approach
1. Build required binaries (`kamn-cli`, `kamn-mcp-server`, `kamn-e2e-harness`).
2. Run sdk-direct live execution for S-01/S-04/S-06 and capture JSON output.
3. Run cli-scripted live execution for S-01/S-04/S-06 and capture JSON output.
4. Run mcp-any live execution for S-01/S-04/S-06 and capture JSON output.
5. Extract per-mode/per-scenario statuses and render deterministic evidence artifact in `docs/research/`.
6. Finalize lifecycle and milestone metadata.

## Affected Modules
- `docs/research/e2e-live-testing-prd-r55-live-probe-execution-evidence.md` (new)
- `specs/5797/{spec.md,plan.md,tasks.md}`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks / Mitigations
- Risk: localhost endpoints unavailable.
  - Mitigation: capture deterministic fail outputs + prerequisite checklist.
- Risk: binary spawn failures for CLI/MCP modes.
  - Mitigation: build binaries locally and pin absolute binary paths in commands.

## Interfaces / Contracts
- Harness run command contract (`kamn-e2e-harness run ...`).
- Live toggles env vars:
  - `KAMN_E2E_SDK_DIRECT_LIVE`
  - `KAMN_E2E_CLI_SCRIPTED_LIVE`
  - `KAMN_E2E_MCP_AGENT_LIVE`

## ADR
- None required.
