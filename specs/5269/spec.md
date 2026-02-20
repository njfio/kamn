# Issue #5269 Spec

- Title: Task: implement M1 anchoring follow-up retry and confirmation policy projection
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Problem Statement
`DataLayerM1AnchoringOrchestrator` now projects deferred/planned/rejected outcomes and persistence metadata, but it does not emit deterministic follow-up policy metadata for retry and confirmation paths. Story `#5250` still requires deterministic and auditable failure/retry behavior.

## Scope
In:
- Add deterministic follow-up policy projection for M1 orchestrator outcomes (`retry`, `poll_confirmation`, `no_retry`).
- Derive follow-up policy from `DataLayerM1AnchorRetryClass` and receipt finality.
- Include stable reason markers and deterministic backoff metadata for retryable in-flight outcomes.
- Add functional/integration/regression tests covering duplicate-pending and conflict-no-retry paths.

Out:
- Runtime daemon loop scheduling threads.
- External job-runner wiring.
- New shell/python/workflow/template surface.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 320
- shell_to_rust_ratio_delta_estimate: -0.0005
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Planned orchestrator outcomes include deterministic follow-up policy metadata with stable reason markers.
- AC-2: Retryable in-flight outcomes project deterministic backoff metadata and retry reason markers.
- AC-3: Conflict/no-retry outcomes project deterministic no-retry policy markers without ambiguous states.
- AC-4: Unit, Functional, Integration, and Regression tests for this slice pass with `fmt` and strict `clippy`.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | new submission + pending/final receipt mix | planned outcome includes follow-up policy metadata |
| C-02 | AC-2 | Functional | duplicate pending receipt (`RetryableInFlight`) | deterministic retry policy + backoff projection |
| C-03 | AC-3 | Regression | rejected/conflict outcome | deterministic no-retry policy + stable reason marker |
| C-04 | AC-1/AC-2 | Integration | orchestrator outcome consumed in adapter-lifecycle flow | persistence plan + follow-up policy remain coherent |
| C-05 | AC-4 | Verification | fmt/clippy + targeted tests | all checks pass |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m1_anchoring_orchestrator`
- `cargo test -p kamn-core --test data_layer_postgres_execution_adapter`
- `cargo test -p kamn-core --test public_api_surface_policy`

## Success Metrics
- M1 runtime orchestration has deterministic, auditable follow-up policy projection for retry/confirmation decisions.
- Story `#5250` AC-3 is covered by concrete conformance tests.
