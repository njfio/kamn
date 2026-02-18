# Tasks — Issue #4862

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: merged child tasks added/ratcheted failing conformance tests in `scripts/framework/test_declarative_policy_checker.py`, `scripts/lib/test_exec_dispatch_registry.sh`, and lane-registry drift contracts.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: merged implementation in #4906/#4907 hardened declarative checker + migration dispatch; lane-registry generation/drift enforcement already in-repo and validated for #4869.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: declarative checker migration to 100 wrappers and registry source-of-truth generation remove manual wrapper/manifest maintenance pathways.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/framework/test_declarative_policy_checker_contract.sh` -> passed
    - `bash scripts/lib/test_exec_dispatch_registry.sh` -> passed
    - `bash scripts/framework/test_lane_registry_generation.sh` -> passed
    - `bash scripts/framework/test_check_lane_registry_drift.sh` -> passed
    - `bash scripts/framework/test_contract_framework.sh` -> passed
