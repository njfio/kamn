# Issue #4043 Spec

- Title: Task: implement local-heavy api compatibility matrix lane with deterministic artifact schema
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Problem Statement
Schema compatibility needs matrix-level evidence with deterministic artifacts so deeper compatibility classes are validated outside CI fast-gate while remaining fail-closed and bounded.

## Scope
In scope:
- Add local-heavy API compatibility matrix lane with deterministic artifact schema and compatibility class projection.
- Add fail-closed policy checker for artifact/marker drift and incompatibility class mismatches.
- Add contract-lane composition for lane + policy + tamper rejection.
- Add Rust tests covering unit/functional/integration/regression/performance conformance.
- Update ops docs markers for local-heavy matrix controls.

Out of scope:
- CI fast-gate governance wiring for the compatibility checker (tracked by #4044).
- Runtime protocol migration tooling.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 850
- rust_loc_delta_estimate: 620
- shell_to_rust_ratio_delta_estimate: +0.0025
- shell_surface_mitigation_issue: #5310

## Acceptance Criteria
- AC-1: Local-heavy matrix lane emits deterministic artifact schema and compatibility-class markers in dry-run and run projections.
- AC-2: Policy checker fails closed on schema drift, marker drift, and incompatibility class mismatches with deterministic reason codes.
- AC-3: Contract lane composes matrix lane + policy checker and rejects deterministic tamper mutations.
- AC-4: Local-heavy run mode is explicit opt-in and bounded; dry-run remains CI-safe and bounded.
- AC-5: Unit, Functional, Integration, Regression, and Performance tests are present and passing.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `run-lane --mode dry-run` | deterministic artifact schema + matrix row markers + `dry_run_no_commands_executed` |
| C-02 | AC-1 | Functional | fixture with compatible + incompatible classes | compatible rows -> `GO/none`; incompatible rows -> class-specific deterministic reason codes |
| C-03 | AC-2 | Functional | valid report to `check-policy` | `status=ok`, `final_decision=GO`, policy status `verified` |
| C-04 | AC-2 | Regression | tampered matrix row status marker | deterministic fail-closed reason code |
| C-05 | AC-3 | Integration | `run-contract-lane` | lane + policy composition markers all verified with `GO` |
| C-06 | AC-4, AC-5 | Performance | dry-run contract lane runtime | bounded within configured threshold |

## Test Mapping
- `cargo test -p kamn-core --test api_compatibility_matrix_local_heavy_contract unit_api_compatibility_matrix_local_heavy_lane_dry_run_emits_deterministic_artifact_schema_markers -- --exact`
- `cargo test -p kamn-core --test api_compatibility_matrix_local_heavy_contract functional_api_compatibility_matrix_local_heavy_lane_projects_compatible_and_incompatible_classes -- --exact`
- `cargo test -p kamn-core --test api_compatibility_matrix_local_heavy_contract functional_api_compatibility_matrix_local_heavy_policy_accepts_valid_report -- --exact`
- `cargo test -p kamn-core --test api_compatibility_matrix_local_heavy_contract regression_api_compatibility_matrix_local_heavy_policy_rejects_tampered_matrix_marker -- --exact`
- `cargo test -p kamn-core --test api_compatibility_matrix_local_heavy_contract integration_api_compatibility_matrix_local_heavy_contract_lane_composes_lane_and_policy -- --exact`
- `cargo test -p kamn-core --test api_compatibility_matrix_local_heavy_contract performance_api_compatibility_matrix_local_heavy_contract_lane_dry_run_stays_within_budget -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_api_compatibility_matrix_local_heavy_markers -- --exact`

## Success Metrics
- Deterministic local-heavy compatibility artifact schema with explicit compatibility-class evidence.
- Fail-closed checker reasons remain stable and machine-parseable.
- Local-heavy run mode remains explicit opt-in and outside fast-gate defaults.
