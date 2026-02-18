# Issue #3632 Spec

- Title: `Story: harden unified API-observability stack contracts and local-heavy governance`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Unified API and observability routing required explicit drift controls, compatibility matrix checks, and local-heavy governance markers to prevent silent regressions.

## Scope
In:
- Compatibility matrix and parity markers for unified API + observability.
- Deterministic fail-closed policy/contract checks.
- CI-fast exclusion governance for local-heavy validation lanes.

Out:
- Re-implementing API migration primitives already delivered.

## Acceptance Criteria
- AC-1: unified API-observability route contracts remain deterministic and fail closed on drift.
- AC-2: local-heavy compatibility lane emits deterministic evidence markers.
- AC-3: CI-fast exclusion policy remains enforced for local-heavy lanes.
- AC-4: command-surface and docs contracts remain in sync.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh` | policy marker checks pass |
| C-02 | AC-1/AC-2 | Conformance | `bash scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh` | contract lane emits deterministic markers |
| C-03 | AC-2 | Functional | `bash scripts/runtime/test_check_service_api_reason_code_compatibility_live_policy.sh` | reason compatibility matrix checks pass |
| C-04 | AC-2 | Functional | `bash scripts/runtime/test_check_service_api_serde_payload_parity_live_policy.sh` | serde payload parity checks pass |
| C-05 | AC-3 | Regression | `bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh` | CI-fast exclusion guard passes |
| C-06 | AC-4 | Conformance | `bash scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh` | command/docs contract markers remain synchronized |

## Test Mapping
- `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
- `scripts/runtime/test_check_service_api_reason_code_compatibility_live_policy.sh`
- `scripts/runtime/test_check_service_api_serde_payload_parity_live_policy.sh`
- `scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh`

## Success Metrics
- Unified stack compatibility markers remain deterministic and fail closed.
- Local-heavy lanes remain out of CI-fast while preserving governance evidence.
