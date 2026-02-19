# Issue #5193 Spec

- Title: Subtask: extend docs-contract data-driven harness pattern to kamn-core and kamn-sdk
- Status: Implemented
- Priority: P2
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Problem Statement
`kamn-node` adopted a shared docs-contract matrix pattern, but equivalent bounded migrations have not been applied in `kamn-core` and `kamn-sdk`. This leaves inconsistent assertion structure and prevents reuse of a single migration playbook across crates.

## Scope
In:
- `kamn-core` bounded slice: consolidate service API docs-contract suites into a shared case-matrix harness:
  - `service_api_contract_docs.rs`
  - `service_api_lifecycle_contract_docs.rs`
- `kamn-sdk` bounded slice: refactor `rust_sdk_alpha_docs.rs` into explicit case-matrix harness structure.
- Remove superseded `kamn-core` one-file suites after parity migration and add migration-contract assertions.
- Document migration checklist guidance in issue template flow for future docs-contract migration subtasks.

Out:
- Full migration of all `kamn-core` docs-contract suites.
- CI workflow/script changes.

## Acceptance Criteria
- AC-1: At least one bounded docs-contract slice in `kamn-core` and `kamn-sdk` is represented via case-matrix harness structures.
- AC-2: Superseded `kamn-core` one-file suites in the selected slice are removed after parity validation.
- AC-3: Migration guidance for future issue intake/templates is documented with explicit docs-contract matrix checklist markers.

## Conformance Cases
| Case | AC | Tier | Input | Expected |
|---|---|---|---|---|
| C-01 | AC-1 | Functional | Shared harness sources in core+sdk slices | Case matrices exist with deterministic case IDs and non-empty marker sets |
| C-02 | AC-2 | Regression | Core migration inventory contract + filesystem | Selected superseded core suites are absent and migration contract passes |
| C-03 | AC-3 | Conformance | Subtask issue template marker contract | Template includes docs-contract migration checklist markers |

## Test Mapping
- C-01 -> `cargo test -p kamn-core --test service_api_docs_contract_harness`
- C-01 -> `cargo test -p kamn-sdk --test rust_sdk_alpha_docs`
- C-02 -> `cargo test -p kamn-core --test service_api_docs_harness_migration_contract`
- C-03 -> `cargo test -p kamn-core --test docs_contract_template_guidance_contract`

## Success Metrics
- Core bounded migration inventory suites are removed and covered by shared harness + migration contract.
- SDK bounded migration slice uses case-matrix structure.
- Template guidance markers exist for future docs-contract migration issue intake.
- Shell LOC delta remains `0`.
