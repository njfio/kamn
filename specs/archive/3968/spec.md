# Spec - Issue #3968

- Title: Task: graduate target missing-docs modules with public API coverage contracts
- Parent: #3965
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

Missing-docs allow-list debt can regress unless module graduations and exemption constraints are enforced by deterministic policy checks and contributor-facing docs markers.

## Objective

Close the task by codifying parent-level AC mapping for completed missing-docs subtasks (`#3974`, `#3975`) and verifying the docs/rustdoc governance contract lanes remain fail-closed.

## Scope

In scope:
- First missing-docs graduation batch coverage from `#3974`.
- Exemption regression reason-marker contracts from `#3975`.
- Parent task AC/conformance mapping and lifecycle artifacts.

Out of scope:
- Full repository docs completion beyond the prioritized batch.
- Feature or protocol behavior changes.

## Acceptance Criteria

- AC-1: Target modules are graduated from missing-docs exemptions with stable public API docs contracts.
- AC-2: Coverage checks fail closed for graduated-module regressions.
- AC-3: Contributor-facing docs/strategy markers reflect graduation governance.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh` and `bash scripts/ci/test_missing_docs_graduation_batch_report_contract.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_missing_docs_velocity_guard_contract.sh` and `bash scripts/ci/test_missing_docs_throughput_report_contract.sh` pass.
- C-03 (AC-3): `cargo test -p kamn-core --test runtime_architecture_docs` and `bash scripts/ci/test_ci_strategy_contract.sh` pass.
- C-04 (AC-4): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes.

## Success Metrics

- Graduation-batch and exemption-regression checks remain deterministic and fail closed.
- Missing-docs governance remains visible in contributor-facing strategy/docs contract surfaces.
