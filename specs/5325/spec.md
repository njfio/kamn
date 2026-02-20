# Issue #5325 Spec

- Title: Decompose `data_layer_m10_partition_archival` root module below monolith threshold
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
`crates/kamn-core/src/data_layer_m10_partition_archival.rs` reached 2,001 LOC in R45 after Phase-6 runtime/scheduler/retry/evidence integration. The file now mixes contract types with multi-domain implementation logic, creating a single high-churn review hotspot.

## Acceptance Criteria
- AC-1: Root `data_layer_m10_partition_archival.rs` is reduced to a facade/type-contract surface (target: <=900 LOC) with implementation logic extracted into child modules.
- AC-2: Public API and behavior remain backward compatible for existing callsites and tests (`kamn_core::data_layer_m10_partition_archival::*`).
- AC-3: Existing M10 conformance suites continue passing without test semantic drift.
- AC-4: `cargo clippy -p kamn-core -- -D warnings` remains clean after decomposition.

## Scope
In scope:
- Extract implementation-heavy sections into `crates/kamn-core/src/data_layer_m10_partition_archival/*.rs`.
- Preserve existing constants/types/function names and reason-code semantics.
- Preserve external module path and re-export surface.

Out of scope:
- Changing M10 behavior semantics.
- Introducing new dependencies.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Structural | LOC scan of root module | root LOC <= 900 and child modules present |
| C-02 | AC-2 | Integration | compile + existing M10/public API tests | no API breakage and no behavior drift |
| C-03 | AC-3 | Conformance | `cargo test -p kamn-core --test data_layer_m10_partition_archival` | existing M10 cases pass unchanged |
| C-04 | AC-4 | Quality | `cargo clippy -p kamn-core -- -D warnings` | zero warnings |

## Test Mapping
- `cargo test -p kamn-core --test data_layer_m10_partition_archival`
- `cargo test -p kamn-core --test public_api_surface_policy`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- Root M10 file exits monolith band (>1K) and reaches <=900 LOC.
- No regressions in M10 conformance/documentation contract suites.
