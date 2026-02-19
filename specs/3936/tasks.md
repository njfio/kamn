# Issue #3936 Tasks

- Issue: #3936
- Status: Implemented

## Ordered Tasks
- [x] T1: retire `unreachable!()` path via child subtask `#3941` (`PR #5153`).
- [x] T2: retire/guard `expect(` path coverage via child subtask `#3940` (`PR #5154`).
- [x] T3: keep runtime watchdog panic-path retirement mapping synchronized.
- [x] T4: verify scoped node/core tests, lint, and formatting checks.
- [x] T5: close parent task after child backlog completion and AC mapping verification.

## Tier Mapping
- Unit: signer decode-failure typed assertion path, extractor fixture regression.
- Functional: startup/API/observability panic-path source guard.
- Integration: runtime watchdog docs contract plus scoped node guard tests.
- Regression: signer-source macro guard and cfg(test)-aware extraction guard.
- Performance: N/A (error-path hardening and test/docs guard updates only).
