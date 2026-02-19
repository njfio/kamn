# Issue #5223 Tasks

- Issue: #5223
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Ordered Tasks
- T1 (Tests/RED): add marker assertions that require wave issue IDs + parseable inventory markers in planning/review docs.
- T2 (Implementation/GREEN): create follow-up migration subtasks for wave A/B/C and capture IDs.
- T3 (Implementation/GREEN): update planning/review docs with typed-DID inventory + migration wave markers.
- T4 (Verification): run targeted docs-contract tests and shell-ratio guardrail checks.
- T5 (Process): update issue process log, set spec status to `Implemented`, prepare PR with AC mapping and shell-surface declaration.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | N/A (no reusable helper library change) |
| Functional | docs marker presence and parseability checks |
| Integration | follow-up issue-link marker integrity checks |
| Regression | fail-closed marker drift tests |
