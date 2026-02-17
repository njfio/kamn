# Spec — Issue #4865

- Title: Task: replace hardcoded dispatcher and wrapper mapping with manifest/registry resolution (phases 1-2)
- Parent: - Parent story: #4860
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Reviewed
- Priority: P1

## Objective

Eliminate static case maps in non-kolme dispatch path and convert tiny wrappers to generated/symlink dispatch.

## Problem Statement

This task executes a bounded plan slice needed to reduce shell surface while preserving deterministic contracts.

## Scope

In scope:
- Issue-defined implementation slice and deterministic behavior contracts.
- Conformance and regression checks mapped to acceptance criteria.
- Shell-surface governance markers where script/workflow surface changes.

Out of scope:
- Unrelated runtime/product features.
- Non-deterministic policy behavior.

## Acceptance Criteria

- AC-1: Dispatch resolution no longer depends on hardcoded lane case blocks.
- AC-2: Wrapper conversion removes duplicated <=8-line scripts with parity checks.
- AC-3: Fallback reason taxonomy/version markers remain stable.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
