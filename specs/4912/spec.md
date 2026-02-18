# Spec — Issue #4912

- Title: Task: execute legacy-script deletion wave, spec archival policy, and hard shell LOC ceiling
- Parent: - Program epic: #3812
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Implemented
- Priority: P1

## Objective

Reverse shell LOC drift by removing superseded legacy shell scripts, archiving completed issue specs, and adding a fail-closed hard shell LOC ceiling gate in CI.

## Problem Statement

Shell governance currently permits transitional growth pressure from legacy compatibility wrappers and unbounded active-spec accumulation.

## Scope

In scope:
- Deletion wave for explicitly superseded legacy shell scripts with migration-contract proof.
- Archive policy and mechanics for completed issue specs.
- CI hard-ceiling checker for repository shell LOC.

Out of scope:
- Runtime feature behavior changes.
- Rust module decomposition work.

## Acceptance Criteria

- AC-1: Legacy shell scripts proven superseded by migration contracts are listed and removed.
- AC-2: Completed issue specs are archived under `specs/archive/` with deterministic pointer metadata.
- AC-3: CI enforces a hard shell LOC ceiling (`130000`) with deterministic reason taxonomy output.
- AC-4: Unit/Functional/Integration/Regression checks for the new controls pass.

## Conformance Cases

- C-01 (AC-1): legacy wrapper files fail an existence contract after migration cleanup.
- C-02 (AC-2): archived spec directories contain complete artifacts and active paths expose archive pointers.
- C-03 (AC-3): shell LOC checker passes below ceiling and fails with deterministic reasons above ceiling.
- C-04 (AC-4): `scripts/ci/test_ci_tools.sh` remains green with new checks wired.

## Success Metrics / Signals

- `tracked_sh_lines_non_symlink` remains below `130000` in CI.
- Legacy wrapper inventory decreases with explicit deletion manifest.
- Completed-issue spec directories are moved out of active `specs/<id>/`.
