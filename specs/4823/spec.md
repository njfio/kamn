# Spec — Issue #4823

- Title: Subtask: replace framework wave wrapper matrix duplicates with wave-definition-driven runner
- Parent: Parent task: #4813
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Collapse duplicated non-Kolme wave10-wave19 lightweight wrapper-matrix tests into a single shared runner driven by wave definition data files while preserving CI command surface compatibility.

## Problem Statement

Ten framework scripts duplicate the same dispatcher/symlink assertions with only wrapper lists changing. This creates maintenance drift and script LOC growth for every new lightweight wave.

## Scope

In scope:
- add a shared wave-driven runner at `scripts/framework/test_non_kolme_wave_lightweight_contract_lane_dispatch_wrapper_matrix.sh`
- add per-wave wrapper definition files under `scripts/framework/wave_definitions/`
- convert `wave10`-`wave19` test entrypoints into symlinks to the shared runner
- add an explicit contract test that verifies shared-runner + symlink topology

Out of scope:
- CI wave budget trend script consolidation (`#4824`)
- JSON helper and test-harness migrations (`#4825`, `#4826`)
- any dispatcher contract/protocol changes outside wave-matrix test consolidation

## Acceptance Criteria

- AC-1: A single shared runner executes wave10-wave19 lightweight wrapper matrix assertions using per-wave definition files.
- AC-2: Existing CI invocations (`test_non_kolme_wave${lightweight_wave}_lightweight_contract_lane_dispatch_wrapper_matrix.sh`) remain valid without command-surface changes.
- AC-3: Wave-matrix duplication is measurably reduced while preserving deterministic unknown-wrapper fallback marker assertions.

## Conformance Cases

- C-01 (AC-1): `scripts/framework/test_non_kolme_wave_lightweight_wrapper_runner_contract.sh` fails when shared runner or wave symlink topology is missing and passes when topology is correct.
- C-02 (AC-2): `for wave in {10..19}; do bash scripts/framework/test_non_kolme_wave${wave}_lightweight_contract_lane_dispatch_wrapper_matrix.sh; done` passes unchanged entrypoint names.
- C-03 (AC-3): `bash scripts/ci/test_ci_tools.sh` passes with wave matrix coverage and deterministic fallback markers preserved.

## Success Metrics / Signals

- Ten duplicated wave scripts converted to symlink entrypoints.
- Wave-specific logic moved to one runner script (`116` lines) and definition data files (`39` lines) instead of ten duplicated bodies (`472` lines previously).
- CI-tools regression suite remains green after consolidation.
