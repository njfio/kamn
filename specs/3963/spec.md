# Spec - Issue #3963

- Title: Epic: R27.7 reduce script-surface sprawl and accelerate docs graduation closure
- Parent: #3812
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

Script-surface sprawl and documentation debt increase operational risk unless migration parity, shell-budget governance, and docs/rustdoc navigation contracts are enforced together at epic scope.

## Objective

Close the R27.7 epic with explicit AC/conformance mapping across child stories:
- `#3964` wrapper-family migration parity/governance,
- `#3965` missing-docs graduation and navigation/rustdoc governance.

## Scope

In scope:
- Wrapper migration parity and shared-dispatch compatibility checks.
- Shell-surface anti-duplication and ratio/ceiling governance checks.
- Missing-docs graduation plus architecture navigation/rustdoc integrity contracts.
- Epic-level lifecycle artifacts and consolidated verification mapping.

Out of scope:
- Full shell-to-Rust replacement in a single epic.
- Feature-level protocol/runtime changes unrelated to this governance tranche.

## Acceptance Criteria

- AC-1: Wrapper-family migration parity checks remain deterministic and fail closed.
- AC-2: Shell-surface budget governance enforces ratio/duplication/ceiling guardrails with deterministic failures.
- AC-3: Missing-docs graduation and navigation/rustdoc governance checks remain deterministic and fail closed.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing for the epic’s contract surfaces.

## Conformance Cases

- C-01 (AC-1): `bash scripts/framework/test_non_kolme_contract_lane_dispatch_wrapper_matrix.sh` and `bash scripts/ci/test_wrapper_dispatch_legacy_entrypoint_compatibility.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_check_script_duplication_budget.sh`, `bash scripts/ci/test_check_shell_rust_ratio_guardrail.sh`, and `bash scripts/ci/test_check_shell_loc_hard_ceiling.sh` pass.
- C-03 (AC-3): `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh`, `bash scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh`, and `cargo test -p kamn-core --test runtime_architecture_docs` pass.
- C-04 (AC-4): `bash scripts/ci/test_ci_strategy_contract.sh` and `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` pass.

## Success Metrics

- Wrapper parity, shell-surface governance, and docs/rustdoc governance remain green and fail-closed in CI fast gate.
- Epic closure is auditable in-repo with deterministic AC->test traceability.
