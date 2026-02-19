# Issue #5193 Tasks

- Issue: #5193
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Ordered Tasks
- T1 (Tests/RED): add core migration inventory contract expecting selected one-file suites to be retired.
- T2 (Implementation/GREEN): add core shared service API docs matrix harness and migrate markers from retired suites.
- T3 (Implementation/GREEN): refactor sdk `rust_sdk_alpha_docs` into a case-matrix harness.
- T4 (Template Guidance/GREEN): add docs-contract migration checklist markers to subtask template and verify via contract test.
- T5 (Verification): run targeted core+sdk docs suites, migration contracts, format, and clippy.
- T6 (Process): update issue process log and PR AC/tier evidence.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | matrix inventory-size and non-empty marker invariants |
| Functional | core+sdk matrix marker checks |
| Conformance | template guidance marker contract |
| Regression | superseded suite retirement contract in core |
