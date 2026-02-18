# Spec — Issue #3973

- Title: Subtask: add wrapper-duplication and script-to-rust ratio guardrail checker for CI-fast governance
- Parent: #3967
- Milestone: R27.7 Script-surface consolidation and docs graduation
- Status: Implemented
- Priority: P1

## Problem Statement

Current CI-fast shell-surface checks enforce duplication budget and trend policy, but they do not provide a dedicated hard guardrail on repository shell-to-rust ratio computed from git-tracked sources.

## Objective

Add a deterministic shell-to-rust ratio guardrail checker and wire it into CI-fast and CI tool regression lanes without breaking existing script-surface governance checks.

## Scope

In scope:
- New checker script for shell-to-rust ratio guardrail.
- Threshold configuration file for warn/fail ratio levels.
- CI-fast workflow step + artifact upload.
- CI contract tests and command-surface wiring updates.
- `docs/ci/strategy.md` policy documentation update.

Out of scope:
- Automatic script deletion or baseline auto-refresh.
- Changes to existing duplication budget semantics.

## Acceptance Criteria

- AC-1: Checker computes shell/rust ratio from git-tracked, non-symlink `*.sh` and `*.rs` files and emits deterministic markers.
- AC-2: Checker fails closed with deterministic reason codes when ratio exceeds fail threshold or config is invalid.
- AC-3: CI-fast runs checker behind script-surface scope gate and uploads the checker report artifact.
- AC-4: CI tool regression contracts validate checker behavior and workflow wiring.
- AC-5: CI strategy docs describe the new ratio guardrail command and report output.

## Conformance Cases

- C-01 (AC-1): pass-path run with permissive threshold returns `status=ok`, `final_decision=GO`, and numeric ratio metrics.
- C-02 (AC-2): fail-path run with strict threshold returns `status=fail`, `final_decision=NO-GO`, and `shell_rust_ratio_fail_threshold_exceeded`.
- C-03 (AC-2): invalid/missing threshold config returns deterministic config error reason code.
- C-04 (AC-3): `.github/workflows/ci-fast-gate.yml` contains guarded checker step and artifact upload for ratio report.
- C-05 (AC-4): `scripts/ci/test_ci_tools.sh` and command-surface contracts include new checker test lane.
- C-06 (AC-5): `docs/ci/strategy.md` includes ratio guardrail command/report references.

## Success Metrics

- CI-fast includes a blocking ratio guardrail decision under script-surface budget checks.
- Ratio guardrail report emits stable schema and reason taxonomy markers.
- No regression in fast CI tools lane.
