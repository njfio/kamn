# E2E Live Testing PRD Phase-6 Runtime Validation Gap Analysis

## Context
This artifact records phase-6 runtime validation execution markers.

## Baseline (Before #5606)
- `phase6_runtime_validation_status_before=partial`
- `phase6_runtime_validation_contract=missing`

## Implemented in #5606
- Added deterministic `runtime_validation_execution` markers to run output:
  - `requested`
  - `orchestration_contract`
  - `lifecycle_contract`
  - `live_validation_contract`
  - `evidence_contract`
  - `overall`
- Validation marker semantics are coherent with external execution state:
  - external disabled -> validation markers `SKIP`
  - external enabled with passing preflight -> validation markers `PASS`

## Status Markers (After #5606)
- `phase6_runtime_validation_contract=implemented`
- `phase6_runtime_validation_status_after=implemented`

## Follow-up Scope
- `phase6_runtime_external_validation_execution_status=complete`

## Extended in #5680
- External execution validation markers now consume probe-derived runtime contract status.
- `orchestration_contract` and `lifecycle_contract` are `FAIL` when probe execution fails.
- `overall` status now deterministically aggregates orchestration, lifecycle, live-validation, and evidence contract statuses.

## Status Markers (After #5680)
- `phase6_runtime_validation_probe_derived_status=implemented`
- `phase6_runtime_validation_probe_failure_contract=implemented`
