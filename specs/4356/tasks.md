# Tasks: #4356 Explicit Key-Source Enforcement and Fallback-Key Rejection

- T1 (Unit/Functional/Conformance): add failing tests for key-source marker/contract-version enforcement and fallback leakage negative proofs.
- T2 (Implementation): enforce explicit key-source contract/version checks and real-node command marker checks in runtime integration policy checker.
- T3 (Implementation): add deterministic key-source/fallback taxonomy output mapping fields in policy checker.
- T4 (Docs): update key-management and release go/no-go checklist markers.
- T5 (Verification): run targeted script tests + repo quality gates.

## Tier Mapping

- Unit: checker validation branches for key-source/fallback marker enforcement.
- Functional: policy pass/fail command-line behavior for synthetic report mutations.
- Conformance: C-01..C-06 mapped to script-level checks.
- Integration: runtime integration contract lane end-to-end policy invocation.
- Regression: fallback-key leakage negative proofs remain fail-closed.
- Performance: script checks remain bounded by existing lane budgets.
