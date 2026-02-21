# Issue #4004 Spec

- Title: Subtask: implement local-heavy load lane runner with deterministic throughput-latency-error markers
- Status: Implemented
- Type: subtask
- Priority: P1
- Milestone: specs/milestones/r27-9-throughput-capacity-and-performance-regression-hardening/index.md

## Problem Statement

The milestone needs a dedicated local-heavy capacity/load runner that emits deterministic throughput,
latency, and error markers for baseline and fault profiles. Existing local-heavy lanes cover adjacent
domains (for example crash-restart and API-compatibility), but not this specific capacity/load profile
contract.

## Scope

In scope:
- Add a local-heavy capacity/load runner contract with deterministic baseline/fault profile outputs.
- Emit a versioned artifact schema + reason taxonomy for fail-closed threshold evaluation.
- Add Rust contract tests (unit/functional/integration/regression/performance).
- Update `docs/ops/configuration.md` with load-profile marker expectations and validation commands.

Out of scope:
- External traffic-generator integration.
- Running local-heavy execution in CI fast-gate default paths.

## Shell-Surface Estimates

- shell_loc_delta_estimate: 320
- rust_loc_delta_estimate: 340
- shell_to_rust_ratio_delta_estimate: -0.0001
- shell_surface_mitigation_issue: None

## Acceptance Criteria

- AC-1: Runner emits stable throughput/latency/error markers with versioned schema/taxonomy markers.
- AC-2: Profile switching behavior is deterministic for `baseline` and `fault`.
- AC-3: Runner fails closed for invalid profile inputs and threshold-breach outcomes.
- AC-4: Unit, Functional, Integration, and Regression tests are present and passing.
- AC-5: Runtime budget/performance guard is present and passing.

## Conformance Cases

| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Unit | `baseline` dry-run invocation | deterministic schema/taxonomy + throughput/latency/error markers; `status=pass` |
| C-02 | AC-2 | Functional | `fault` dry-run invocation | deterministic marker projection with fail-closed `NO-GO` decision |
| C-03 | AC-2 | Integration | run-mode invocation with/without local opt-in | explicit local-only opt-in gate enforced; run-mode command markers deterministic |
| C-04 | AC-3 | Regression | invalid profile value | deterministic fail-closed error marker and non-zero exit |
| C-05 | AC-5 | Performance | baseline lane invocation | completes within bounded runtime budget |

## Test Mapping

- `cargo test -p kamn-core --test local_heavy_capacity_load_lane_contract -- --nocapture`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs service_api_ops_configuration_contains_local_heavy_capacity_load_lane_markers -- --exact`

## Success Metrics / Observable Signals

- Local-heavy load lane outputs deterministic throughput/latency/error markers for baseline/fault profiles.
- Fault profile deterministically produces `NO-GO` threshold-breach reason markers.
- Docs and docs-contract tests remain synchronized with runner markers.
