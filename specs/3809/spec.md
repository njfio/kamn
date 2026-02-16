# Issue #3809 Spec

- Title: `Subtask: expand unified API-observability compatibility matrix and parity markers`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
The unified service API + observability route compatibility lane has limited matrix coverage and does not emit explicit parity checkpoint markers, making route-level drift detection less deterministic.

## Scope
In:
- Expand compatibility matrix rows to cover additional service API and observability route classes.
- Emit deterministic parity/fail-closed checkpoint markers in lane and policy outputs.
- Add regression checks for route mismatch and marker drift fail-closed behavior.
- Update CI/runtime docs with expanded matrix and marker contracts.

Out:
- Internet-scale load testing.
- New network dependencies.

## Acceptance Criteria
- AC-1: Given the route compatibility lane, when matrix rows are generated, then service API and observability route classes include health, metrics, readiness/stream, and negative-path coverage with deterministic row IDs.
- AC-2: Given lane and policy execution, when the report is validated, then parity/fail-closed checkpoint markers are emitted and policy enforces them fail-closed.
- AC-3: Given tampered compatibility reports, when route values or checkpoint markers drift, then policy rejects with deterministic reason codes.
- AC-4: Given contract documentation surfaces, when compatibility lane contracts are described, then expanded matrix coverage and checkpoint markers are documented.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional/Conformance | `bash scripts/runtime/test_validate_service_api_observability_route_compatibility_live.sh` | Expanded matrix row count and route-class rows are emitted and validated |
| C-02 | AC-2 | Functional/Conformance | `bash scripts/runtime/test_check_service_api_observability_route_compatibility_live_policy.sh` | Parity/fail-closed checkpoint markers validated in policy pass path |
| C-03 | AC-3 | Regression/Conformance | `bash scripts/runtime/test_check_service_api_observability_route_compatibility_live_policy.sh` tamper subcases | Route mismatch and checkpoint marker drift reject with deterministic reason codes |
| C-04 | AC-4 | Integration/Docs | `bash scripts/runtime/test_validate_service_api_observability_route_compatibility_live_contract_lane.sh` | Contract lane + docs parity markers stay verified |

## Test Mapping
- `scripts/runtime/service_api_observability_route_compatibility_contract.py`
- `scripts/runtime/test_validate_service_api_observability_route_compatibility_live.sh`
- `scripts/runtime/test_check_service_api_observability_route_compatibility_live_policy.sh`
- `scripts/runtime/test_validate_service_api_observability_route_compatibility_live_contract_lane.sh`
- `docs/ci/strategy.md`
- `docs/architecture/service-runtime.md`

## Success Metrics
- Matrix coverage and parity/fail-closed markers are deterministic.
- Tampered route or marker drift is rejected with stable reason code output.
- Runtime compatibility lane tests remain green in dry-run policy/contract paths.
