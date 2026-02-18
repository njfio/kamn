# Issue #3643 Spec

- Title: `Task: enable TLS on observability routes with endpoint compatibility coverage`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
Observability endpoints needed TLS parity with service endpoints so operational telemetry surfaces are not left in plaintext.

## Scope
In:
- TLS checks and compatibility coverage for observability endpoints.
- Deterministic fail-closed behavior for invalid TLS configurations.
- Endpoint contract-lane validation.

Out:
- New observability schema design.

## Acceptance Criteria
- AC-1: observability endpoints are validated under TLS with deterministic behavior.
- AC-2: observability route semantics remain compatible under TLS.
- AC-3: fail-closed negative-path behavior is enforced for TLS mismatch/misconfiguration.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-2 | Functional | `bash scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh` | observability TLS policy checks pass |
| C-02 | AC-1/AC-2 | Conformance | `bash scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh` | observability TLS contract lane passes |
| C-03 | AC-3 | Regression | `bash scripts/ci/test_check_transport_observability_tls_ci_smoke_convergence.sh` | TLS observability smoke convergence checks pass |
| C-04 | AC-3 | Functional | `bash scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh` | fail-closed marker taxonomy remains enforced |

## Test Mapping
- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
- `scripts/ci/test_check_transport_observability_tls_ci_smoke_convergence.sh`

## Success Metrics
- Observability TLS contracts remain deterministic and fail closed.
