# Tasks — Issue #4827

- [x] T1 (Red): add failing contract test before implementation.
  Evidence: `bash scripts/framework/test_declarative_policy_checker_contract.sh` failed with `expected declarative policy checker to be executable`.
- [x] T2 (Green): implement checker and schema validation/output behavior.
  Evidence: `scripts/framework/declarative_policy_checker.py` added; contract test now passes.
- [x] T3 (Refactor): centralize policy evaluation semantics and operator handling in one module with reusable functions.
- [x] T4 (Verify): run deterministic suites and record results.
  Evidence:
  - `python3 scripts/framework/test_declarative_policy_checker.py`
  - `bash scripts/framework/test_declarative_policy_checker_contract.sh`
  - `bash scripts/framework/test_contract_framework.sh`
