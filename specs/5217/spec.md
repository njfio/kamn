# Issue #5217 Spec

- Title: Task: Consolidate doc-contract suites below 100 test files
- Status: Implemented
- Priority: P1
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
R43 identified fragmented docs-contract coverage spread across many singleton `include_str!` test files. The current shape keeps behavior deterministic but increases maintenance and file-surface overhead.

## Scope
In:
- Consolidate a bounded wave of singleton docs-contract suites into one matrix harness in `crates/kamn-core/tests`.
- Preserve all existing marker assertions via case-matrix entries.
- Retire superseded singleton suites and add migration regression assertions for retired file inventory.
- Keep shell surface LOC-neutral while rewiring any lane wrappers that referenced retired singleton test binaries.

Out:
- Changing documentation marker policies.
- Refactoring non-doc test lanes.
- New shell/python/workflow lanes or policy expansions.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 140
- shell_to_rust_ratio_delta_estimate: -0.0007
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Docs-contract file count is reduced measurably from baseline.
- AC-2: A shared harness matrix covers all retired singleton test assertions.
- AC-3: Migration regression checks prove retired singleton files are removed.
- AC-4: Full targeted docs-contract commands stay green with no shell LOC increase.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Count docs-contract test files before/after consolidation | Post-change count decreases |
| C-02 | AC-2 | Functional | Run consolidated harness matrix tests | All migrated marker checks pass |
| C-03 | AC-3 | Regression | Run migration regression check for retired files | Regression test fails if retired files reappear |
| C-04 | AC-4 | Conformance | Run targeted docs-contract suites + clippy/fmt checks | All pass, shell delta remains zero |

## Test Mapping
- C-01 -> `rg --files crates/kamn-core/tests | rg '_docs\\.rs$|docs_contract' | wc -l`
- C-02 -> `cargo test -p kamn-core --test docs_contract_matrix_wave2_harness`
- C-03 -> `cargo test -p kamn-core --test docs_contract_matrix_wave2_migration_contract`
- C-04 -> `cargo test -p kamn-core --test docs_contract_matrix_wave2_harness --test docs_contract_matrix_wave2_migration_contract` and `cargo clippy -p kamn-core --tests -- -D warnings`

## Success Metrics
- Net reduction in singleton docs-contract file count.
- All migrated marker assertions preserved in matrix format.
- No shell LOC additions for this task.
