# Issue #4315 Spec

- Title: `Subtask: add red tests for async api concurrency limit breaches and fail-closed backpressure behavior`
- Status: `Implemented`
- Priority: `P1`
- Milestone: `specs/milestones/r27-30-async-api-runtime-networked-peer-transport-and-durable-block-pipeline-governance/index.md`
- Parent: `#4311`

## Problem Statement
Async API limiter behavior exists, but the open subtask still requires explicit conformance coverage that proves concurrency/backpressure breaches fail closed with deterministic reason outputs.

## Scope
In:
- Add deterministic concurrency-limit breach regression coverage for repeated bounded-concurrency rounds.
- Add deterministic projection coverage for backpressure reason-code mappings.
- Add ops configuration docs markers for async API backpressure failure modes and guard them with docs tests.

Out:
- API endpoint redesign.
- Runtime limiter algorithm changes.

## Acceptance Criteria
- AC-1: Concurrency-limit breach checks fail closed with `429` and stable reason code outputs.
- AC-2: Backpressure rejection reason projections remain deterministic/checker-consumable.
- AC-3: Regression coverage preserves deterministic limiter behavior across repeated rounds.
- AC-4: `docs/ops/configuration.md` includes async API backpressure failure-mode markers.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Integration | `cargo test -p kamn-node integration_service_api_endpoint_rejects_when_concurrency_limit_is_exceeded -- --exact` | at least one bounded request is accepted and at least one request fail-closes on concurrency pressure |
| C-02 | AC-1, AC-3 | Regression | `cargo test -p kamn-node regression_service_api_endpoint_concurrency_limit_reason_code_stays_stable_across_rounds -- --exact` | repeated concurrency-pressure rounds keep `429` + `service_api_ingress_concurrency_limit_exceeded` |
| C-03 | AC-2 | Functional | `cargo test -p kamn-node functional_service_api_endpoint_backpressure_projection_covers_reason_codes -- --exact` | backpressure reason codes map to stable projection fields |
| C-04 | AC-4 | Docs | `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_async_backpressure_failure_modes -- --exact` | ops configuration doc contains async backpressure marker/taxonomy text |

## Test Mapping
- `crates/kamn-node/src/main_tests/service_api_endpoint_tests.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Success Metrics
- Fail-closed backpressure reason outputs remain deterministic across repeated load fixtures.
- Docs contracts for async API backpressure failure modes are test-enforced.
