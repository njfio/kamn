# Issue #3937 Tasks

- Issue: #3937
- Status: Implemented

## Ordered Tasks
- [x] T1: implement scoped panic-path policy checker hardening (`#3942`, `PR #5156`).
- [x] T2: add docs-contract marker/remediation parity tests (`#3943`, `PR #5157`).
- [x] T3: verify checker/docs-contract suites and guardrails stay green.
- [x] T4: close parent task with full AC/conformance lineage.

## Tier Mapping
- Unit: checker cfg(test)-prefix regression fixture and docs marker assertions.
- Functional: policy checker reason-code outputs and docs marker presence.
- Integration: checker harness + `ci_strategy_docs` suite composition.
- Regression: fail-closed fixtures for checker blind spots and docs drift.
- Performance: low-cost checker/docs validation paths remain bounded.
