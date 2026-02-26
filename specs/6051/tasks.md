# Tasks: Issue #6051

## Ordered Tasks
- T1 (RED): Add failing script tests for frozen-doc change rejection and manifest fail-closed behavior.
- T2 (GREEN): Implement review-document freeze checker with schema-versioned report.
- T3 (GREEN): Wire checker into `ci-fast-gate` pull-request steps and artifact upload.
- T4 (GREEN): Update CI command/workflow contract tests for checker presence.
- T5 (GREEN): Update `docs/ci/strategy.md` and `ci_strategy_docs` marker assertions.
- T6 (VERIFY): Run targeted script/workflow/docs tests and `cargo fmt --all --check`.

## Tier Mapping
- Unit: T1, T2
- Functional: T2, T4
- Conformance: T1, T3, T5, T6
- Integration: T3, T4, T6
- Regression: T1, T6
- Performance: N/A (small manifest + changed-file list)
