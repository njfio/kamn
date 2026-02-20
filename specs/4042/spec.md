# Issue #4042 Spec

- Title: Task: implement request-response schema compatibility checker for supported version pairs
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Problem Statement
Supported API version pairs need deterministic compatibility checks so breaking request/response schema drift fails closed before merge.

## Scope
In scope:
- Add deterministic request-response schema compatibility lane with fixture-driven pair evaluation.
- Add fail-closed policy checker for incompatible schema deltas and taxonomy drift.
- Add contract-lane composition with tamper rejection and docs parity checks.
- Add Rust tests covering unit/functional/integration/regression/performance conformance.
- Add strategy/ops docs markers for checker commands and reason taxonomy.

Out of scope:
- Semantic protocol migration planning.
- Runtime payload migration tooling.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 560
- shell_to_rust_ratio_delta_estimate: -0.0010
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Compatibility lane emits deterministic fixture schema and supported-pair compatibility markers.
- AC-2: Policy checker fails closed on incompatible pair projection, schema drift, and marker drift with deterministic reason codes.
- AC-3: Contract-lane composition validates tamper rejection and strategy/ops docs parity markers.
- AC-4: Unit, Functional, Integration, Regression, and Performance checks pass under CI-safe dry-run expectations.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `run-lane --mode dry-run` | deterministic report schema + compatible/incompatible markers + dry-run reason code |
| C-02 | AC-1 | Functional | fixture row with breaking response field removal | row evaluates to `NO-GO` with deterministic incompatible reason |
| C-03 | AC-2 | Functional | valid report to `check-policy` | `status=ok`, `final_decision=GO`, policy marker status `verified` |
| C-04 | AC-2 | Regression | tampered report marker | deterministic fail-closed reason code |
| C-05 | AC-3 | Integration | `run-contract-lane` | lane+policy+docs parity all verified with GO final decision |
| C-06 | AC-4 | Performance | dry-run contract lane runtime | bounded within configured threshold |

## Test Mapping
- `cargo test -p kamn-core --test request_response_schema_compatibility_contract unit_request_response_schema_compatibility_lane_dry_run_emits_deterministic_markers -- --exact`
- `cargo test -p kamn-core --test request_response_schema_compatibility_contract functional_request_response_schema_compatibility_lane_includes_compatible_and_incompatible_fixture_rows -- --exact`
- `cargo test -p kamn-core --test request_response_schema_compatibility_contract functional_request_response_schema_compatibility_checker_accepts_valid_report -- --exact`
- `cargo test -p kamn-core --test request_response_schema_compatibility_contract regression_request_response_schema_compatibility_checker_rejects_tampered_marker -- --exact`
- `cargo test -p kamn-core --test request_response_schema_compatibility_contract integration_request_response_schema_compatibility_contract_lane_composes_policy_and_docs_parity -- --exact`
- `cargo test -p kamn-core --test request_response_schema_compatibility_contract performance_request_response_schema_compatibility_contract_lane_dry_run_stays_within_budget -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_runtime_request_response_schema_compatibility_contract_lane_ci_mode_markers -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_request_response_schema_compatibility_markers -- --exact`

## Success Metrics
- Deterministic supported-version-pair compatibility enforcement with explicit fail-closed taxonomy.
- CI strategy and ops docs remain parity-enforced with Rust docs-contract tests.
- No new shell script body growth while extending compatibility governance coverage.
