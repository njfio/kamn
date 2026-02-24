# Tasks: Issue #5840

## Ordered Tasks
- [x] T1 (RED): add failing regression fixtures for brace-heavy cfg(test) strings/raw strings in checker and docs-contract parser tests.
- [x] T2 (GREEN): implement literal/comment-aware cfg(test) item skipper in `check_no_production_expect.py`.
- [x] T3 (GREEN): implement matching cfg(test) skipper hardening in `review_r53_docs_contract.rs`.
- [x] T4 (GREEN): align production expect inventory formula marker text with implemented semantics.
- [x] T5 (VERIFY): run checker and docs-contract suites to confirm deterministic pass/fail behavior.

## Tier Mapping
- Unit: cfg(test) skipper parser-state transitions.
- Functional: production expect inventory and checker contract status.
- Regression: brace-heavy cfg(test) fixtures and top-level cfg(test) import + production expect path.
- Integration: CI checker harness + docs-contract suite execution.
