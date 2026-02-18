# Spec - Issue #3969

- Title: Task: publish architecture navigation artifacts and rustdoc build contract checks
- Parent: #3965
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

Documentation discoverability regresses when architecture navigation artifacts and rustdoc integrity checks are not continuously enforced by deterministic contract lanes.

## Objective

Close the parent task with explicit AC/conformance mapping for completed child subtasks (`#3976`, `#3977`) covering navigation marker checks and low-cost rustdoc governance lanes.

## Scope

In scope:
- Architecture navigation artifact marker contracts delivered in `#3976`.
- Rustdoc smoke/policy contract lanes delivered in `#3977`.
- Parent task lifecycle artifacts and verification traceability.

Out of scope:
- External documentation hosting/publishing platform changes.
- Non-documentation feature work.

## Acceptance Criteria

- AC-1: Navigation artifacts are published/updated with deterministic marker checks.
- AC-2: Rustdoc checks fail closed on build or marker drift.
- AC-3: CI-fast includes low-cost documentation governance checks.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `cargo test -p kamn-core --test runtime_architecture_docs` and `cargo test -p kamn-core --test kolme_runtime_architecture_docs` pass.
- C-02 (AC-2): `bash scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh` and `bash scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh` pass.
- C-03 (AC-3): `bash scripts/ci/test_ci_strategy_contract.sh` and `bash scripts/ci/test_ci_tools_command_surface_contract.sh` pass.
- C-04 (AC-4): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes.

## Success Metrics

- Architecture navigation markers and rustdoc artifact checks remain deterministic and fail closed.
- Docs governance checks continue to run within fast-mode CI tooling lanes.
