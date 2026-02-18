# Issue #3650 Spec

- Title: `Task: validate unified API-observability stack compatibility and performance`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Unified API-observability serving needed deterministic compatibility checks and local-heavy governance evidence to prevent regressions.

## Scope
In:
- Compatibility matrix checks for unified API + observability paths.
- Deterministic local-heavy lane evidence and policy checks.
- CI-fast exclusion governance for local-heavy lane.

Out:
- Internet-scale load testing.

## Acceptance Criteria
- AC-1: unified stack compatibility matrix checks pass deterministically.
- AC-2: local-heavy lane emits deterministic evidence markers.
- AC-3: CI-fast exclusions remain enforced for local-heavy lane.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | `bash scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh` | compatibility policy checks pass |
| C-02 | AC-1/AC-2 | Conformance | `bash scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh` | unified stack contract lane passes |
| C-03 | AC-1 | Functional | `bash scripts/runtime/test_check_service_api_reason_code_compatibility_live_policy.sh` | reason-code compatibility matrix passes |
| C-04 | AC-1 | Functional | `bash scripts/runtime/test_check_service_api_serde_payload_parity_live_policy.sh` | serde parity matrix passes |
| C-05 | AC-3 | Regression | `bash scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh` | CI-fast exclusion policy check passes |

## Test Mapping
- `scripts/runtime/test_check_unified_api_observability_local_heavy_live_policy.sh`
- `scripts/runtime/test_validate_unified_api_observability_local_heavy_live_contract_lane.sh`
- `scripts/runtime/test_check_service_api_reason_code_compatibility_live_policy.sh`
- `scripts/runtime/test_check_service_api_serde_payload_parity_live_policy.sh`
- `scripts/ci/test_unified_api_observability_local_heavy_ci_exclusion_policy.sh`

## Success Metrics
- Unified compatibility markers and CI-governance checks are deterministic and green.
