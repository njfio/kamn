# Issue #5188 Tasks

- Issue: #5188
- Milestone: specs/milestones/r27-46-r42-gap-remediation-and-maintainability-closure/index.md

## Ordered Tasks
- T1 (Tests/RED): add `crates/kamn-core/tests/public_api_surface_policy.rs` with expected schema/policy behavior and failing assertions for missing fixtures/contracts.
- T2 (Fixtures/GREEN): add baseline + threshold fixtures and minimal parser/policy logic to satisfy AC-1/AC-2.
- T3 (Docs/GREEN): update `docs/ci/strategy.md` and docs-contract tests for baseline refresh and waiver workflow (AC-3).
- T4 (Refactor): tighten helper functions, reason markers, and deterministic ordering.
- T5 (Verify): run fmt, clippy, targeted tests, and relevant crate tests for regression confidence.
- T6 (Process): update issue status/comments, open PR with AC mapping + TDD evidence, merge, and mark spec `Implemented`.

## Test Tier Mapping
| Tier | Planned Coverage |
|---|---|
| Unit | scanner helpers inside integration test module (pure functions) |
| Functional | report generation and policy status checks |
| Conformance | schema markers + deterministic per-module fields + baseline/threshold contract |
| Regression | fail-closed unchecked-growth path and waiver validation path |
