# Spec - Issue #3964

- Title: Story: migrate remaining wrapper families to manifest dispatcher with parity guarantees
- Parent: #3963
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

Wrapper-family duplication and per-lane shell growth increase maintenance overhead unless migration to shared dispatcher/manifest paths is enforced with deterministic parity and budget contracts.

## Objective

Complete wrapper-family migration story outcomes by combining migration parity contracts and shell-surface governance checks under one auditable story-level specification.

## Scope

In scope:
- Migration/parity outcomes delivered by task `#3966` (including subtasks `#3970`, `#3971`).
- Shell-surface governance outcomes delivered by task `#3967`.
- Story-level AC/conformance mapping for regression and integration validation.

Out of scope:
- Replacing all shell tooling with Rust in one story.
- Non-governance feature development unrelated to migration/parity contracts.

## Acceptance Criteria

- AC-1: Prioritized wrapper families are migrated with parity-preserving shared dispatcher paths.
- AC-2: Budget and duplication checks fail closed on shell-surface regression.
- AC-3: CI governance remains bounded with deterministic fast-gate contract lanes.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh` and `bash scripts/ci/test_wrapper_dispatch_legacy_entrypoint_compatibility.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_check_script_duplication_budget.sh`, `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`, and `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh` pass.
- C-03 (AC-3): `bash scripts/ci/test_run_fast_gate_budget_delta_contract_lane.sh` and `bash scripts/ci/test_ci_tools_command_surface_contract.sh` pass.
- C-04 (AC-4): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes.

## Success Metrics

- Wrapper-family migration parity and legacy compatibility contracts remain deterministic in fast gate.
- Shell-surface governance checks continuously enforce anti-duplication and shell/rust budget guardrails.
