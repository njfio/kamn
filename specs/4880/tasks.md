# Tasks — Issue #4880

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence:
    - `python3 scripts/framework/test_declarative_policy_checker.py` failed with `ContractError not raised` for invalid `reason_taxonomy_version` marker.
    - `bash scripts/framework/test_declarative_policy_checker_contract.sh` failed with `expected checker to fail on invalid reason_taxonomy_version marker format`.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: `scripts/framework/declarative_policy_checker.py` now fail-closes invalid taxonomy markers with deterministic validation (`reason_taxonomy_version` must match `<namespace>.v<integer>`).
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: added explicit schema/taxonomy contract section for declarative checker in `docs/ci/strategy.md` and aligned unit + shell contract tests to the same marker rule.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `python3 scripts/framework/test_declarative_policy_checker.py` -> passed
    - `bash scripts/framework/test_declarative_policy_checker_contract.sh` -> passed
    - `bash scripts/framework/test_contract_framework.sh` -> passed
    - `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh` -> passed
