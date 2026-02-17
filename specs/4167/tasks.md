# Tasks: #4167 Fallback Prohibition and Signer-Material RED Coverage

- T1 (RED/Conformance): add failing checks for fallback signer prohibition and missing signer-material mapping.
- T2 (GREEN): update preflight policy tests to assert deterministic mapping outputs.
- T3 (Docs): update ops configuration contracts for explicit signer-material/fallback error expectations.
- T4 (Verification): run targeted policy and docs contract tests.

## Tier Mapping

- Unit: policy checker fixture mutation assertions for signer-config/fallback classes.
- Functional: preflight policy checker fail-closed behavior.
- Integration: docs contracts and checker composition tests.
- Conformance: C-01..C-03.
- Regression: fallback-path reintroduction guard.
