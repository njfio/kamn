# Tasks: #4169 Quorum Marker Parity and Tamper Rejection Tests

- T1 (RED/Conformance): add failing assertions for quorum approval-count parity mismatch and custody tamper rejection reason markers.
- T2 (GREEN): update deploy preflight checker tests to assert deterministic marker outputs for C-01/C-02.
- T3 (Docs): add deployment compatibility runbook markers for quorum parity/tamper contract.
- T4 (Verification): run targeted script/docs tests and record outputs in PR.

## Tier Mapping

- Unit: checker fixture mutation assertions for quorum and custody marker logic.
- Functional: policy checker CLI fail-closed behavior for tampered reports.
- Integration: deployment preflight checker + docs compatibility contract tests.
- Conformance: C-01..C-03.
- Regression: marker taxonomy stability and tamper rejection drift.
