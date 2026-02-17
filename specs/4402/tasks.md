# Tasks — #4402

Status: Reviewed

T1 (RED)
- Add failing policy/lane tests for fuzz seed replay drift, concurrency race misclassification, and CI/local boundary marker drift.

T2 (GREEN)
- Implement deterministic reason mapping and CI smoke/local-heavy boundary marker enforcement in invariant-fuzz-concurrency checker/lane summary.

T3 (Regression)
- Re-run invariant-fuzz-concurrency policy and contract-lane suites; verify deterministic pass/fail outputs.

T4 (Docs)
- Update `docs/ci/strategy.md` with invariant-fuzz-concurrency boundary marker/reason contracts.
