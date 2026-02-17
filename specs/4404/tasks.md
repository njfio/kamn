# Tasks — #4404

Status: Reviewed

T1 (RED)
- Add failing tests for telemetry evidence-link completeness and partial-evidence acceptance drift.

T2 (GREEN)
- Enforce evidence-link completeness and evidence-convergence checks in telemetry policy checker.
- Preserve run-mode evidence artifacts so links are valid for policy verification.

T3 (Regression)
- Re-run telemetry policy + contract-lane suites and verify deterministic reason markers on pass/fail paths.

T4 (Docs)
- Update `docs/ci/strategy.md` telemetry lane section with new fail-closed reason markers and boundary governance wording.

