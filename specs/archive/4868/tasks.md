# Tasks — Issue #4868

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: `python3 scripts/framework/test_declarative_policy_checker.py` failed on new invalid taxonomy marker conformance case before validator hardening.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: `scripts/framework/declarative_policy_checker.py` now fail-closes `reason_taxonomy_version` to `<namespace>.v<integer>` and supports deterministic declarative checks for migrated wrappers.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: `scripts/lib/exec_dispatch.py` migration wave expanded to 100 `check_*`/`validate_*` wrappers; registry ratchet enforced in `scripts/lib/test_exec_dispatch_registry.sh`.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `python3 scripts/framework/test_declarative_policy_checker.py` -> passed
    - `bash scripts/framework/test_declarative_policy_checker_contract.sh` -> passed
    - `bash scripts/lib/test_exec_dispatch_registry.sh` -> passed
    - `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh` -> passed
