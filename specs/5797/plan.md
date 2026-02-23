# Plan: Issue #5797 - Execute Live Harness S-01/S-04/S-06 Across SDK/CLI/MCP Modes

- Issue: #5797
- Status: Implemented (agent-authored; human review requested in PR)
- Spec: `specs/5797/spec.md`

## Approach
1. Build/verify required binaries (`kamn-node`, `kamn-cli`, `kamn-mcp-server`, `kamn-e2e-harness`) and start a local API runtime.
2. Execute live harness runs for `sdk-direct`, `cli-scripted`, and `mcp-tau` with `S-01,S-04,S-06`.
3. Capture per-mode JSON outputs under `.tmp/5797-live/` and extract deterministic scenario status markers.
4. Publish a research evidence artifact with commands, outcomes, and blocker details if needed.
5. Finalize milestone metadata and lifecycle artifacts, then run targeted non-regression docs-contract checks.

## Affected Modules and Artifacts
- `docs/research/e2e-live-testing-prd-r55-live-harness-5797-execution-evidence.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/5797/spec.md`
- `specs/5797/plan.md`
- `specs/5797/tasks.md`

## Risks and Mitigations
- Risk: local service prerequisites unavailable.
  - Mitigation: capture exact blocker diagnostics (command, error marker, missing prerequisite) and keep evidence fail-closed.
- Risk: spec-volume non-regression cap breach from adding issue lifecycle artifacts.
  - Mitigation: apply preserve-spec-cap offset by pruning one legacy implemented spec directory.
- Risk: mode-specific auth/scope drift.
  - Mitigation: use explicit chain context env vars and mode-specific live toggles consistently across all runs.

## Verification Strategy
- Conformance commands from `specs/5797/spec.md` C-01..C-06.
- Targeted docs-contract gates for R50/R53 non-regression cap validation after spec-cap offset.
