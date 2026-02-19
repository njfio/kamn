# Issue #3790 Tasks

- Issue: #3790
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add docs-contract assertions for transient classifier and retry schedule markers.
- [x] T2 (Green): update runtime-commit architecture docs with transient classifier + bounded schedule table.
- [x] T3 (Unit/Functional): run helper classifier/backoff/decision tests and verify deterministic behavior.
- [x] T4 (Regression): run malformed fail-fast regression, docs-contract tests, fmt/clippy, and shell guardrails.
- [ ] T5 (Verify): open/merge PR and close issue with DoD markers.

## Tier Mapping
- Unit: retry classifier/backoff helper tests in `runtime_kolme_live.rs`.
- Functional: helper decision-matrix examples via `retry_decision_for_attempt` tests.
- Integration: N/A (helper-only scope; documented per issue contract).
- Regression: malformed fail-fast helper test + docs-contract drift checks + lint/guardrails.
- Performance: N/A (no runtime path expansion beyond existing bounded schedule helpers).
