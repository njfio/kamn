# Issue 6250 Spec

Status: Reviewed
Priority: P1
Milestone: R59 Swarm Gap Closure
Parent: #6246

## Problem Statement
Top-10 follow-up tracking for shell-surface governance must be closed with current, measured evidence. The repository needs a deterministic guardrail that enforces shell/rust ratio non-regression and a concrete migration of a PR-critical shell lane into Rust test coverage.

## Scope
In scope:
- Measure current shell/rust ratio with the fast-gate guardrail and keep it below 1.0.
- Move one high-frequency CI tool regression lane from shell test wrapper coverage to Rust test coverage.
- Keep shell-rust ratio gate deterministic and fail-closed in fast-gate.
- Update R59 follow-up documentation with measured before/after values.

Out of scope:
- Rewriting all CI shell wrappers.
- Changing workflow trigger topology or replacing the shell-rust ratio checker itself.

## Acceptance Criteria
- AC-1: Measured shell-to-rust ratio remains below 1.0 after the migration.
- AC-2: Fast-gate continues to enforce deterministic shell-surface non-regression via existing guardrail checkers.
- AC-3: At least one PR-critical CI tool regression lane is migrated from shell test wrapper coverage to Rust integration test coverage.
- AC-4: Unit, Functional, Integration, and Regression test evidence is present for the migrated lane.

## Conformance Cases
- C-01 (AC-1, Functional): `bash scripts/ci/check_shell_rust_ratio_guardrail.sh --threshold-file .ci/shell-rust-ratio-guardrail.env --output-json <file>` reports `shell_to_rust_ratio < 1.0`.
- C-02 (AC-2, Conformance): `.github/workflows/ci-fast-gate.yml` still runs `check_shell_rust_ratio_guardrail.sh` and shell-surface trend/ratchet checks.
- C-03 (AC-3, Integration): `scripts/ci/test_ci_tools.sh` fast-mode command surface includes `cargo test -p kamn-core --test ci_shell_rust_ratio_guardrail_contract` and no longer includes `test_check_shell_rust_ratio_guardrail.sh`.
- C-04 (AC-4, Unit/Regression): `cargo test -p kamn-core --test ci_shell_rust_ratio_guardrail_contract` verifies pass/warn/fail/validation reason-code behavior for the checker wrapper.
- C-05 (AC-4, Regression): `bash scripts/ci/test_ci_tools_command_surface_contract.sh` passes with updated command-surface expectations.

## Success Metrics
- Shell/rust ratio remains below 1.0 with positive fail-threshold headroom.
- Shell LOC decreases and Rust LOC increases for the migrated lane.
- PR-critical CI tool regression path retains deterministic reason-code markers.
