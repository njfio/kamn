# Issue #3788 Tasks

- Issue: #3788
- Status: Done

## Ordered Tasks
- [x] T1 (Red): add docs-contract assertions for route parity matrix + fail-closed drift markers.
- [x] T2 (Green): update `docs/observability/contracts.md` with explicit route parity marker contract section.
- [x] T3 (Functional/Integration): run baseline and secure-mode route parity runtime tests.
- [x] T4 (Regression): run unknown-path and malformed-method fail-closed route tests.
- [x] T5 (Verify): run fmt/clippy/shell guardrails and merge PR.

## Tier Mapping
- Unit: parity marker vocabulary assertions in docs-contract tests.
- Functional: endpoint parity semantics across observability surfaces.
- Integration: live baseline + TLS-required route parity probes.
- Regression: unknown-path and malformed-method fail-closed checks + docs drift assertions.
- Performance: N/A (no runtime-path expansion in this issue).
