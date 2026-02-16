# Issue #4316 Spec

- Title: `Subtask: implement async lifecycle limiter and deterministic api rejection reason projection`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4311`

## Problem Statement
Async lifecycle limiter paths already reject requests under pressure, but rejection reason projection is scattered and not explicitly taxonomy-governed for checker consumers.

## Scope
In:
- Add deterministic lifecycle-limiter rejection projection for async API reason codes.
- Route middleware limiter/backpressure rejection responses through a shared projection mapping.
- Add conformance tests for projection determinism and live limiter integration behavior.
- Add docs contract markers at `docs/service/api-contract.md` for lifecycle rejection taxonomy and projection matrix.

Out:
- External queueing system rollout.
- Public endpoint redesign.

## Acceptance Criteria
- AC-1: lifecycle limiter reason projection is deterministic for limiter/backpressure reason codes.
- AC-2: async lifecycle limiter fail-closed responses use stable projection metadata (status/error/outcome class).
- AC-3: integration coverage validates projection over live concurrency-limiter rejection behavior.
- AC-4: docs contract contains lifecycle rejection taxonomy version and projection matrix markers.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `cargo test -p kamn-node unit_service_api_endpoint_lifecycle_rejection_projection_is_deterministic -- --exact` | projection for lifecycle reason codes is stable across repeated lookups |
| C-02 | AC-2 | Functional | `cargo test -p kamn-node functional_service_api_endpoint_lifecycle_rejection_projection_maps_limiter_classes -- --exact` | limiter reason codes map to deterministic rejection class/status/outcome |
| C-03 | AC-3 | Integration | `cargo test -p kamn-node integration_service_api_endpoint_lifecycle_projection_matches_live_concurrency_rejection -- --exact` | live concurrency rejection reason projects to expected limiter class |
| C-04 | AC-2 | Regression | `cargo test -p kamn-node regression_service_api_endpoint_lifecycle_projection_sender_suspension_class_stays_stable -- --exact` | sender suspension rejection remains in stable projection class |
| C-05 | AC-2 | Performance | `cargo test -p kamn-node performance_service_api_endpoint_lifecycle_projection_loop_stays_within_local_budget -- --exact` | projection loop remains bounded while preserving deterministic output |
| C-06 | AC-4 | Docs | `cargo test -p kamn-core --test service_api_lifecycle_contract_docs service_api_contract_contains_async_lifecycle_rejection_taxonomy_markers -- --exact` | docs retain lifecycle rejection taxonomy markers and matrix |

## Test Mapping
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `crates/kamn-node/src/service_api_endpoint.rs`
- `docs/service/api-contract.md`
- `crates/kamn-core/tests/service_api_lifecycle_contract_docs.rs`

## Success Metrics
- Lifecycle limiter reason mapping is checker-consumable via deterministic projection.
- Live concurrency-limit rejection remains fail-closed and projection-consistent.
- Docs-contract markers fail closed via docs tests.
