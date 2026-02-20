# Issue #5257 Spec

- Title: Task: implement PostgreSQL repository bridge contracts and RLS session projection
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
`#5255` introduced baseline migration scaffolding and marker checks, but there is still no repository bridge contract layer that maps validated M0/M2 inputs into deterministic SQL execution descriptors. This prevents Phase-1 story `#5248` from progressing toward live PostgreSQL integration.

## Scope
In:
- Add a `kamn-core` module that projects deterministic PostgreSQL operation descriptors for insert/query/search paths.
- Validate requester DID session context for RLS execution metadata.
- Project M2 default RLS policies into deterministic SQL statement descriptors.
- Add contract tests for mapping determinism and fail-closed invalid-input behavior.

Out:
- Live DB connectivity, pools, and query execution runtime.
- Adding/upgrading dependencies (`sqlx` integration deferred to a follow-up slice).
- Extension adapters and higher data-layer phases.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 220
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: M0-backed insert operations project to deterministic PostgreSQL SQL descriptors with explicit bind-order markers.
- AC-2: Query and blind-index search operations project deterministic SQL descriptors that carry validated requester session context.
- AC-3: M2 default RLS policy templates project to deterministic SQL statement descriptors with stable ordering.
- AC-4: Invalid requester DID/session inputs fail closed with structured non-panicking error variants.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | valid `DataLayerM0EnvelopeRecord` + owner/requester DIDs | insert descriptor uses stable SQL and bind markers |
| C-02 | AC-2 | Functional | query/search requests | deterministic descriptor SQL + requester session metadata |
| C-03 | AC-3 | Integration | `data_layer_m2_default_rls_policies()` output | deterministic statement projection order and SQL fragments |
| C-04 | AC-4 | Regression | invalid requester DID and invalid search inputs | structured fail-closed errors with reason codes |

## Test Mapping
- C-01/C-02/C-03/C-04:
  - `cargo test -p kamn-core --test data_layer_postgres_repository_bridge`
- Regression guard:
  - `cargo test -p kamn-core --test data_layer_m2_gateway_access`

## Success Metrics
- Bridge contracts provide deterministic SQL descriptors for Phase-1 repository wiring.
- RLS requester session metadata is type-validated at the bridge boundary.
- Invalid input paths are deterministic, non-panicking, and regression-tested.
