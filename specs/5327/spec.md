# Issue #5327 Spec

- Title: Resume doc-contract harness consolidation to reduce include_str test-file count below 100
- Status: Reviewed (agent-authored; human review requested in PR)
- Type: task
- Priority: P1
- Milestone: specs/milestones/r27-program-operational-hardening-and-live-validation/index.md

## Problem Statement
R45 reports doc-contract suites stalled at 122+ files using `include_str!()`-centric one-file-per-doc patterns. This increases maintenance overhead and review noise.

## Acceptance Criteria
- AC-1: Reduce doc-contract include_str test-file count from current baseline (`123`) to `<100`.
- AC-2: Preserve existing assertion semantics for migrated files (no marker-coverage loss).
- AC-3: Consolidated harness file(s) remain deterministic and compile/test clean.
- AC-4: `cargo clippy -p kamn-core -- -D warnings` remains clean.

## Scope
In scope:
- Consolidate selected small doc-contract files into a shared harness container file under `crates/kamn-core/tests/`.
- Retire migrated `include_str!` standalone implementations after content is preserved in harness modules; keep thin compatibility wrappers where needed for stable lane command-surface and ratio-policy compliance.

Out of scope:
- Rewriting large doc-contract suites.
- Changing docs contract semantics.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Structural | include_str file count scan | count < 100 |
| C-02 | AC-2 | Functional | run consolidated harness tests | migrated assertions pass unchanged |
| C-03 | AC-3 | Integration | run selected doc-contract suites | no regressions from consolidation |
| C-04 | AC-4 | Quality | strict clippy on kamn-core | zero warnings |

## Test Mapping
- `rg -l "include_str!\(" crates/kamn-core/tests crates/kamn-node/tests crates/kamn-sdk/tests | wc -l`
- `cargo test -p kamn-core --test docs_contract_wave3_harness`
- `cargo test -p kamn-core --test service_api_ops_configuration_docs`
- `cargo clippy -p kamn-core -- -D warnings`

## Success Metrics
- include_str test-file count reduced to <100.
- Consolidated harness passes with preserved assertions.
