# Tasks: Issue #6045

## Ordered Tasks
- T1 (RED): Add failing contract expectations for production-target clippy command scope (`--lib --bins`) and fail-closed behavior.
- T2 (GREEN): Implement production-target checker wrapper script + command-surface contract test.
- T3 (GREEN): Wire checker into fast CI tools lane and PR fast-gate workflow.
- T4 (GREEN): Update CI strategy docs and doc-contract assertions for production-target scope markers.
- T5 (VERIFY): Run targeted checker/workflow/doc contract tests and collect pass evidence.

## Tier Mapping
- Functional: T2, T3
- Conformance: T1, T4, T5
- Regression: T1, T5
- Integration: T3
