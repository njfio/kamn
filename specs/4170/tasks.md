# Tasks: #4170 Deterministic Custody Reason Mapping Contract

- T1 (RED/Conformance): add failing assertions for missing custody taxonomy output markers.
- T2 (GREEN): implement custody taxonomy constants/output projection and deterministic mismatch mapping assertions.
- T3 (Docs): update release checklist with custody reason mapping gate markers.
- T4 (Integration/Regression): add docs-contract tests and run targeted checker/docs suites.

## Tier Mapping

- Unit: helper projection ordering for custody reason taxonomy values.
- Functional: deployment preflight checker output markers and fail-closed reasons.
- Integration: checklist/deploy compatibility docs contract tests.
- Conformance: C-01..C-03.
- Regression: custody mismatch mapping order stability.
