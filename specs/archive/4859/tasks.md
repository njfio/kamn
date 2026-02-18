# Tasks — Issue #4859

- [x] T1 (Red): add or update failing tests mapped to conformance cases before implementation.
  - Evidence: phase-aligned contracts were introduced/ratcheted across dispatcher migration, harness/json-helper adoption, declarative checker migration, and lane-registry drift enforcement.
- [x] T2 (Green): implement minimal changes to satisfy all acceptance criteria.
  - Evidence: all milestone child stories/tasks/subtasks (phases 0-7 and sustainment governance) were implemented and merged.
- [x] T3 (Refactor): reduce duplication and improve maintainability while preserving deterministic outputs.
  - Evidence: shell surface consolidated through shared runners/helpers, registry-driven artifacts, and declarative checker dispatch with fail-closed policy contracts.
- [x] T4 (Verify): run required test tiers, capture evidence, and update process log/issue status.
  - Evidence:
    - `bash scripts/ci/test_ci_tools.sh` -> passed
    - `bash scripts/lib/test_exec_dispatch_registry.sh` -> passed
    - `bash scripts/framework/test_declarative_policy_checker_contract.sh` -> passed
    - `bash scripts/framework/test_lane_registry_generation.sh` -> passed
    - `bash scripts/framework/test_check_lane_registry_drift.sh` -> passed
