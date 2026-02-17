# Spec — Issue #4822

- Title: Subtask: implement wrapper exec registry dispatcher and replace <=8-line wrappers with symlinks
- Parent: Parent task: #4812
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Introduce a single exec-dispatch wrapper model that shrinks tiny wrapper shell LOC while preserving deterministic contract-lane behavior.

## Problem Statement

Tiny wrapper scripts were duplicating `exec ...` boilerplate across hundreds of files, inflating shell LOC and making lane wiring harder to maintain.

## Scope

In scope:
- add shared dispatcher entrypoints and registry metadata
- migrate eligible tiny wrappers to symlink-to-dispatcher form
- update affected contract tests from file-content assertions to symlink+registry assertions
- keep CI contract suites deterministic and green after migration

Out of scope:
- non-wrapper architectural refactors
- protocol/wire-format behavior changes

## Acceptance Criteria

- AC-1: Shared dispatcher exists and is executable: `scripts/lib/exec_dispatch.sh`, `scripts/lib/exec_dispatch.py`, `scripts/lib/exec_registry.json`.
- AC-2: Eligible tiny wrappers are migrated to symlinks that resolve to `scripts/lib/exec_dispatch.sh`, with deterministic registry metadata that preserves prior target/args behavior.
- AC-3: CI/runtime/frontend/sdk/compliance contract tests that previously asserted wrapper source content now validate symlink+registry semantics.
- AC-4: Trend budget checks remain valid under symlink wrappers (symlink-aware LOC accounting; baseline fixtures updated where required).
- AC-5: End-to-end CI tools regression suite remains green.

## Conformance Cases

- C-01 (AC-1/AC-2): `bash scripts/lib/test_exec_dispatch_registry.sh` passes.
- C-02 (AC-3): wrapper matrix/contract tests for migrated lanes pass (runtime, sdk, frontend, compliance, ci contract lanes).
- C-03 (AC-4): `bash scripts/ci/test_check_non_kolme_wave_trend_test_loc_soft_budget.sh` passes with symlink-aware LOC semantics.
- C-04 (AC-5): `bash scripts/ci/test_ci_tools.sh` passes.

## Success Metrics / Signals

- Dispatcher registry test passes with deterministic wrapper resolution checks.
- CI tool regression suite exits 0 after migration.
- Wrapper tests assert stable contracts (symlink target + registry entry) instead of brittle inline script text.
