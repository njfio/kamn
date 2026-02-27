# Tasks: Issue #6141

## Ordered Tasks
- T1 (RED/Conformance): Capture failing check showing fast-gate lacks explicit coverage-guided fuzz lane/report command.
- T2 (GREEN/Implementation): Add fast-gate coverage-guided parser fuzz contract lane step + artifact upload.
- T3 (Regression): Extend `test_workflow_scope_policy.sh` assertions for lane/report wiring while preserving deep-lane exclusion checks.
- T4 (Verify): Run scoped workflow-policy and shell checks to validate CI contract behavior.
- T5 (Closure): Map ACs to commands/tests and publish evidence in issue/PR.

## Tier Mapping
- Unit: N/A (workflow/shell contract scope)
- Functional: T2, T4
- Integration: N/A (no runtime integration changes)
- Regression: T3, T4
- Conformance: T1, T2, T3, T4, T5
