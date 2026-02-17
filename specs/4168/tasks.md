# Tasks: #4168 Deterministic Signer Config Error Mapping

- T1 (RED/Conformance): add failing assertions for missing signer-config taxonomy markers and missing/invalid signer-material mapping.
- T2 (GREEN): implement signer-config taxonomy outputs and deterministic mapping in checker.
- T3 (Docs): add signer-config fail-closed contracts to ops configuration docs.
- T4 (Integration/Regression): add/update docs-contract tests and run targeted suites.

## Tier Mapping

- Unit: deterministic signer-config reason projection helper behavior.
- Functional: checker fail-closed mapping for missing/invalid signer material.
- Integration: docs-contract + checker composition tests.
- Conformance: C-01..C-04.
- Regression: fallback prohibition and signer-config mapping drift guards.
