# Tasks — #4222 Admission Decision Taxonomy + Runbook Parity

Status: Implemented

## Ordered Tasks
1. T1 (Red, Regression/Integration): add failing assertions/tamper fixtures for admission decision taxonomy markers and runbook parity divergence in service-api axum policy/lane tests.
2. T2 (Green, Functional): emit admission decision taxonomy markers in validation summary output and stdout markers.
3. T3 (Green, Functional/Regression): enforce admission decision taxonomy checks in policy checker with deterministic fail-closed reasons.
4. T4 (Green, Integration): wire lane required markers, summary propagation, and runbook marker checks for admission decision taxonomy parity.
5. T5 (Green, Docs): update strategy/runbook/release checklist/ops configuration marker references and corresponding docs-contract tests.
6. T6 (Verify, Regression): run targeted shell and Rust docs-contract test commands and capture outputs for PR evidence.

## Tier Mapping
- Unit: targeted marker validation helpers via policy/lane checker tests.
- Functional: baseline GO-path marker validation for summary/policy.
- Conformance: C-01..C-07 via script and docs-contract tests.
- Integration: contract-lane report/policy/runbook composition checks.
- Regression: tampered taxonomy/runbook divergence fixtures.
- Performance: CI smoke boundary unchanged; no new heavy lanes.
