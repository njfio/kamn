# E2E Live Testing PRD Phase-6 Runtime Integration Gap Analysis

## Context
This artifact records phase-6 runtime external execution integration markers.

## Baseline (Before #5600)
- `phase6_runtime_integration_status_before=partial`
- `phase6_runtime_integration_guard_contract=missing`

## Implemented in #5600
- Added explicit guarded run flag: `--enable-external-execution`.
- Added deterministic `runtime_external_execution` object in run output:
  - `requested`
  - `guard_status`
  - `execution_mode`
  - `preflight`
- Added deterministic preflight failures for missing runtime binaries:
  - Kolme binary path existence required when external execution is enabled.
  - Agent binary path existence required for MCP modes when external execution is enabled.

## Status Markers (After #5600)
- `phase6_runtime_integration_guard_contract=implemented`
- `phase6_runtime_integration_status_after=implemented`

## Follow-up Scope
- `phase6_runtime_external_process_orchestration_status=pending`

## Extended in #5680
- External execution path now performs real binary probes (`--help` invocation) for configured runtime binaries.
- Probe outcomes are exposed in `runtime_external_execution.guard_status` and `runtime_external_execution.probe_detail`.
- Probe failures produce deterministic `FAIL` status markers without panic.

## Status Markers (After #5680)
- `phase6_runtime_external_probe_execution_contract=implemented`
- `phase6_runtime_external_probe_failure_status=implemented`
