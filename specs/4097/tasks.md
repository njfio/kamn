# Issue #4097 Tasks

## Ordered Task List
- [x] T1 (Tests/RED): Add overload docs parity contract tests in `ci_strategy_docs.rs` and `service_api_ops_configuration_docs.rs`; run targeted exact tests expecting failure before docs updates.
- [x] T2 (Implementation): Add overload docs parity + go/no-go markers and remediation map to `docs/ci/strategy.md`.
- [x] T3 (Implementation): Add overload remediation marker references in `docs/ops/configuration.md`.
- [x] T4 (GREEN): Run targeted docs-contract tests and fix any drift failures.
- [x] T5 (Regression): Run focused `kamn-core` docs-contract suites for nearby sections to ensure no collateral regressions.
- [ ] T6 (Closure): Update issue status/comments with TDD evidence and shell-surface actuals; open PR with AC/test matrix mapping.

## Tier Mapping
- Unit: reason-code parsing/remediation coverage checks (`ci_strategy_docs.rs`)
- Functional: docs marker presence tests (`ci_strategy_docs.rs`, `service_api_ops_configuration_docs.rs`)
- Integration: strategy/ops/runner parity tests (`ci_strategy_docs.rs`)
- Regression: remediation-per-reason drift tests (`ci_strategy_docs.rs`)
- Performance: N/A (docs/test-only change)
