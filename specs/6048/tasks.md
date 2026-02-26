# Tasks: Issue #6048

## Ordered Tasks
- T1 (RED): Add failing shell contract tests for governance-ratio pass/fail and unknown-prefix fail-closed behavior.
- T2 (GREEN): Implement `check_governance_feature_commit_ratio.py` classifier, ratio enforcement, and JSON report output.
- T3 (GREEN): Wire checker into `scripts/ci/test_ci_tools.sh` fast lane and enforce command-surface contract assertions.
- T4 (GREEN): Add `ci-fast-gate` workflow step + artifact upload; extend workflow policy tests.
- T5 (GREEN): Update `docs/ci/strategy.md` and `ci_strategy_docs` assertions.
- T6 (VERIFY): Run targeted script/workflow/doc tests and clippy/fmt checks for changed surfaces.

## Tier Mapping
- Unit: T1, T2
- Functional: T2, T3
- Conformance: T1, T4, T5, T6
- Integration: T4, T6
- Regression: T1, T6
- Performance: N/A (bounded commit stream input)
