# Issue #4041 Spec

- Title: Task: implement api version-policy checker with supported-window fail-closed enforcement
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-12-api-schema-evolution-and-compatibility-governance/index.md

## Problem Statement
API version windows need deterministic enforcement so unsupported versions fail closed with stable reason taxonomy and docs parity. Without a concrete checker contract, version drift can silently pass release gates.

## Scope
In scope:
- Add a deterministic API version-policy lane report generator with supported/unsupported fixture evaluation.
- Add fail-closed policy checker for supported-window contracts.
- Add contract-lane composition with tamper rejection and docs parity checks.
- Add Rust tests covering unit/functional/integration/regression/performance contracts.
- Add strategy/ops docs markers for checker commands and fail-closed reasons.

Out of scope:
- Multi-major migration tooling.
- Request/response payload compatibility matrix deep lane (tracked by `#4042`..`#4052`).

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 520
- shell_to_rust_ratio_delta_estimate: -0.0015
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Version-policy lane emits deterministic fixture schema and supported-window evaluation markers.
- AC-2: Policy checker fails closed on unsupported-window, schema drift, and marker drift with deterministic reason codes.
- AC-3: Contract-lane composition validates tamper rejection and strategy/ops docs parity markers.
- AC-4: Unit, Functional, Integration, Regression, and Performance checks pass under CI-safe dry-run expectations.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `run-lane --mode dry-run` | deterministic report schema + supported-window markers + dry-run reason code |
| C-02 | AC-1 | Functional | fixture row with unsupported version | row evaluates to `NO-GO` with deterministic unsupported reason |
| C-03 | AC-2 | Functional | valid report to `check-policy` | `status=ok`, `final_decision=GO`, policy marker status `verified` |
| C-04 | AC-2 | Regression | tampered report marker | deterministic fail-closed reason code |
| C-05 | AC-3 | Integration | `run-contract-lane` | lane+policy+docs parity all verified with GO final decision |
| C-06 | AC-4 | Performance | dry-run contract lane runtime | bounded within configured threshold |

## Test Mapping
- `cargo test -p kamn-core --test api_version_policy_contract unit_api_version_policy_lane_dry_run_emits_deterministic_markers -- --exact`
- `cargo test -p kamn-core --test api_version_policy_contract functional_api_version_policy_lane_includes_supported_and_unsupported_fixture_rows -- --exact`
- `cargo test -p kamn-core --test api_version_policy_contract functional_api_version_policy_checker_accepts_valid_report -- --exact`
- `cargo test -p kamn-core --test api_version_policy_contract regression_api_version_policy_checker_rejects_tampered_marker -- --exact`
- `cargo test -p kamn-core --test api_version_policy_contract integration_api_version_policy_contract_lane_composes_policy_and_docs_parity -- --exact`
- `cargo test -p kamn-core --test api_version_policy_contract performance_api_version_policy_contract_lane_dry_run_stays_within_budget -- --exact`
- `cargo test -p kamn-core --test ci_strategy_docs doc_contains_api_version_policy_checker_markers -- --exact`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_api_version_policy_markers -- --exact`

## Success Metrics
- Deterministic supported-window policy enforcement with explicit fail-closed taxonomy.
- CI strategy and ops docs remain parity-enforced with Rust docs-contract tests.
- No new shell script body growth while adding compatibility governance coverage.
