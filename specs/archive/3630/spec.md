# Issue #3630 Spec

- Title: `Story: add TLS termination for service and observability endpoints`
- Status: `Implemented`
- Priority: `P0`
- Milestone: `specs/milestones/r26-2-service-edge-hardening/index.md`

## Problem Statement
TLS capability existed in parts of the runtime surface, but complete production hardening required deterministic TLS behavior across both service and observability endpoints plus release-governance evidence.

## Scope
In:
- TLS behavior coverage for service and observability endpoints.
- Deterministic fail-closed policy and contract-lane markers.
- Go/no-go evidence integration and CI-fast exclusion governance for local-heavy lanes.

Out:
- External certificate issuance automation.
- Cloud ingress provisioning.

## Acceptance Criteria
- AC-1: service and observability endpoint TLS checks are deterministic and fail closed.
- AC-2: release go/no-go evidence includes TLS markers.
- AC-3: local-heavy TLS validations remain enforced while excluded from CI-fast.
- AC-4: route compatibility contracts remain green under TLS.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1/AC-4 | Functional | `bash scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh` | service API TLS policy checks pass |
| C-02 | AC-1/AC-4 | Functional | `bash scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh` | observability TLS policy checks pass |
| C-03 | AC-1/AC-4 | Conformance | `bash scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh` | service TLS contract lane passes |
| C-04 | AC-1/AC-4 | Conformance | `bash scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh` | observability TLS contract lane passes |
| C-05 | AC-2 | Integration | `bash scripts/deploy/test_generate_gonogo_evidence_bundle.sh` | go/no-go evidence bundle includes TLS markers |
| C-06 | AC-2/AC-3 | Integration | `bash scripts/deploy/test_run_gonogo_evidence_contract_lane.sh` | go/no-go contract lane enforces TLS evidence |
| C-07 | AC-3 | Regression | `cargo test -p kamn-core --test release_gonogo_checklist_docs` | docs/governance contract remains synchronized |

## Test Mapping
- `scripts/runtime/test_check_service_api_axum_ingress_live_policy.sh`
- `scripts/runtime/test_check_runtime_observability_endpoint_live_policy.sh`
- `scripts/runtime/test_validate_service_api_axum_ingress_live_contract_lane.sh`
- `scripts/runtime/test_validate_runtime_observability_endpoint_live_contract_lane.sh`
- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Success Metrics
- TLS coverage is deterministic across service + observability.
- Go/no-go surfaces enforce TLS evidence fail-closed.
