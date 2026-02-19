# Issue #3784 Tasks

- Issue: #3784
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add failing docs-contract assertion for local observability artifact schema declarations.
- [x] T2 (Green): update `docs/ci/strategy.md` local observability section with explicit schema markers.
- [x] T3 (Functional): run local observability lane shell contract tests.
- [x] T4 (Regression): run targeted docs-contract + fmt + clippy + shell guardrails.
- [x] T5 (Verify): open/merge PR and close issue with DoD markers.

## Tier Mapping
- Unit: N/A (no new helper or runtime algorithm added).
- Functional: local observability lane shell contract tests.
- Integration: `ci_strategy_docs` local observability docs-contract test.
- Regression: lint + shell-surface governance gates.
