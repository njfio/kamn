# Tasks — Issue #4881

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: `bash scripts/lib/test_exec_dispatch_registry.sh` failed after migration-ratchet test updates with `expected delegated checker execution marker delegate_env=1 in dispatcher output` for `validate_*` wrappers.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: expanded declarative migration eligibility in `scripts/lib/exec_dispatch.py` to include `check_*` and `validate_*` wrappers targeting python contract modules up to 1500 LOC.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: migration contract now enforces first-wave floor of 100 eligible wrappers and documents residual backlog in `docs/research/2026-02-18-declarative-policy-migration-telemetry.md`.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/lib/test_exec_dispatch_registry.sh` -> passed
    - `bash scripts/framework/test_declarative_policy_checker_contract.sh` -> passed
    - `KAMN_CI_TOOLS_FAST_MODE=true timeout 900 bash scripts/ci/test_ci_tools.sh` -> passed
