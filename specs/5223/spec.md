# Issue #5223 Spec

- Title: Task: Plan typed-DID migration wave for non-data-layer String DID callsites
- Status: Implemented
- Priority: P2
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Problem Statement
Typed DID adoption is still uneven outside integrated data-layer modules. Non-data-layer modules continue to expose raw DID `String` boundaries, which preserves type-safety drift and duplicate DID validation logic.

## Scope
In:
- Produce an explicit non-data-layer DID-string inventory snapshot with deterministic markers.
- Define bounded migration waves and link actionable follow-up implementation issues.
- Add/extend docs-contract tests that fail closed on marker drift and invalid issue-link formats.
- Update planning/review docs with migration-progress markers.

Out:
- Executing the full typed-DID migrations in runtime/product code.
- New shell/python/workflow surfaces.

## Shell-Surface Estimates
- shell_loc_delta_estimate: 0
- rust_loc_delta_estimate: 60
- shell_to_rust_ratio_delta_estimate: -0.0003
- shell_surface_mitigation_issue: None

## Acceptance Criteria
- AC-1: Inventory identifies remaining non-data-layer typed-DID migration targets.
- AC-2: Follow-up implementation issues are linked for each migration wave.
- AC-3: Marker schema is compatible with docs-contract assertions.
- AC-4: Shell LOC remains unchanged for this planning task.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Planning/review docs marker block | Inventory marker keys, module counts, and callsite counts are present and parseable |
| C-02 | AC-2 | Integration | Migration wave marker issue list | Every marker issue reference is in `#<id>` format and resolvable to created follow-up issues |
| C-03 | AC-3 | Regression | docs-contract tests for typed-DID markers | Tests fail closed on missing schema markers or malformed issue references |
| C-04 | AC-4 | Conformance | diff-level shell/rust accounting | shell delta remains zero |

## Test Mapping
- C-01/C-02/C-03:
  - `cargo test -p kamn-core --test data_layer_prd_standalone_decision_docs_contract`
  - `cargo test -p kamn-core --test typed_did_migration_backlog_review_docs_contract`
- C-04:
  - `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`
  - diff evidence in PR shell-surface declaration

## Success Metrics
- Non-data-layer typed-DID migration inventory is encoded in deterministic marker lines.
- Migration wave issue IDs are present in both planning and review artifacts.
- Marker drift is guarded by Rust docs-contract tests.
