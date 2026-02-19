# Issue #3791 Tasks

- Issue: #3791
- Status: In Progress

## Ordered Tasks
- [x] T1 (Red): add docs-contract assertions for retry validation commands and deterministic marker declarations.
- [x] T2 (Green): update `docs/planning/kolme-devnet-ops.md` with missing deterministic retry validation contract markers.
- [x] T3 (Functional/Integration): run runtime retry marker and retry exhaustion integration tests.
- [x] T4 (Regression/Performance): run regression + bounded performance retry tests, fmt/clippy, and shell guardrails.
- [ ] T5 (Verify): open/merge PR and close issue with DoD markers.

## Tier Mapping
- Unit: retry helper behavior remains covered by existing runtime helper tests (`runtime_kolme_live.rs`).
- Functional: integrated retry marker behavior and terminal-decision projection tests.
- Integration: mock Kolme transport fault matrix tests validating request-loop retries.
- Regression: retry exhaustion terminal marker tests + docs drift checks + lint/guardrails.
- Performance: bounded retry recovery budget test remains enforced.
