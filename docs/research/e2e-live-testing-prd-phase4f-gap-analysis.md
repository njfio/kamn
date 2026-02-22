# E2E Live Testing PRD Phase-4f Gap Analysis

## Context
This artifact records phase-4f mode-aware lifecycle population contract markers for `kamn-e2e-harness`.

## Baseline (Before #5574)
- `phase4f_status_before=partial`
- `phase4f_mode_aware_rules=missing`
- `phase4f_controlled_fail_path=missing`

## Implemented in #5574
- Added deterministic mode-aware lifecycle rules:
  - `[MCP modes]` AGENT_DEPLOY step records are `SKIP` in non-MCP modes.
  - `[MCP modes]` AGENT_DEPLOY step records are `PASS` in MCP modes.
- Added deterministic controlled fail-path marker behavior:
  - run configurations with `evidence_dir` containing `fail-path` set
    `Verify KAMN Service API health (/healthz)` step to `FAIL`
  - INFRA_UP phase status propagates to `FAIL` for that controlled path

## Status Markers (After #5574)
- `phase4f_mode_aware_rules=implemented`
- `phase4f_controlled_fail_path=implemented`
- `phase4f_status_after=implemented`

## Follow-up Scope
- `phase4g_real_runtime_process_execution_status=pending`
- `phase4h_ci_live_lane_wiring_status=pending`
