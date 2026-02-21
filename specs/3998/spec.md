# Issue #3998 Spec

- Title: Task: implement local-heavy capacity-load lane with deterministic throughput-latency-error artifacts
- Status: Implemented
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement

Local-heavy capacity validation requires deterministic throughput/latency/error artifact projection plus
fail-closed taxonomy enforcement so release/governance checks stay reproducible without public-network load tests.

## Scope

In scope:
- Local-heavy load lane runner contract with deterministic baseline/fault profile markers.
- Fail-closed policy/taxonomy checks for capacity dry-run governance.
- Config/docs marker contracts for load profiles and reason taxonomy parity.

Out of scope:
- Public-network load execution.

## Acceptance Criteria

- AC-1: Load lane produces deterministic artifact schema and throughput/latency/error markers.
- AC-2: Failure taxonomy is explicit and policy-verifiable.
- AC-3: Budget and profile controls are configurable and documented.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | local-heavy runner `baseline` profile dry-run | deterministic schema/taxonomy + throughput/latency/error markers; `status=pass` |
| C-02 | AC-1 | Functional | local-heavy runner `fault` profile dry-run | deterministic fail-closed `NO-GO` threshold breach markers |
| C-03 | AC-2 | Integration | capacity dry-run governance checker with report/workflow/docs contracts | deterministic reason taxonomy ordering and fail-closed reason projection |
| C-04 | AC-3 | Regression | docs-marker drift for runner/policy markers | docs contract tests fail closed on missing markers |
| C-05 | AC-4 | Performance | local-heavy runner bounded invocation | runtime stays within defined budget |

## Test Mapping

- `cargo test -p kamn-core --test local_heavy_capacity_load_lane_contract -- --nocapture`
- `cargo test -p kamn-core --test capacity_ci_dry_run_governance_contract -- --nocapture`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_local_heavy_capacity_load_lane_markers -- --exact`
- `cargo test -p kamn-core --test observability_stack_docs doc_contains_capacity_ci_dry_run_threshold_reason_taxonomy_contract -- --exact`

## Success Metrics / Observable Signals

- Baseline/fault load profiles emit deterministic markers and stable schema/taxonomy versions.
- Capacity policy checker reason taxonomy remains explicit, ordered, and fail-closed.
- Ops/observability docs remain synchronized with runner/policy marker contracts.
