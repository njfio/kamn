# Spec - Issue #3965

- Title: Story: graduate missing-docs allow-list modules and improve architecture navigability contracts
- Parent: #3963
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

API discoverability and documentation integrity degrade when missing-docs graduation and architecture navigation/rustdoc contracts are tracked only at subtask scope without durable story-level conformance mapping.

## Objective

Complete story closure by codifying AC/conformance mapping across child tasks `#3968` (missing-docs graduation) and `#3969` (navigation + rustdoc governance).

## Scope

In scope:
- Missing-docs graduation and exemption regression governance from `#3968`.
- Architecture navigation and rustdoc governance contracts from `#3969`.
- Story-level lifecycle artifacts and verification traceability.

Out of scope:
- Full docs platform redesign or external publishing workflow changes.
- Non-documentation runtime/protocol changes.

## Acceptance Criteria

- AC-1: Missing-docs allow-list graduation contracts remain fail-closed for target modules.
- AC-2: Architecture navigation and rustdoc integrity checks remain deterministic and fail closed.
- AC-3: CI-fast retains low-cost docs governance coverage.
- AC-4: Unit/Functional/Integration/Regression coverage remains present and passing.

## Conformance Cases

- C-01 (AC-1): `bash scripts/ci/test_check_kamn_core_missing_docs_policy.sh` and `bash scripts/ci/test_missing_docs_graduation_batch_report_contract.sh` pass.
- C-02 (AC-2): `bash scripts/ci/test_run_kamn_core_rustdoc_artifact_contract_lane.sh`, `bash scripts/ci/test_check_kamn_core_rustdoc_artifact_policy.sh`, and `cargo test -p kamn-core --test runtime_architecture_docs` pass.
- C-03 (AC-3): `bash scripts/ci/test_ci_strategy_contract.sh` and `bash scripts/ci/test_ci_tools_command_surface_contract.sh` pass.
- C-04 (AC-4): `KAMN_CI_TOOLS_FAST_MODE=true bash scripts/ci/test_ci_tools.sh` passes.

## Success Metrics

- Missing-docs graduation and rustdoc/navigation governance remain deterministic and discoverable at story scope.
- CI-fast retains comprehensive docs governance checks without regression.
