# Spec — Issue #4860

- Title: Story: deliver shared shell substrate and dispatch/wrapper reduction phases (0-2)
- Parent: - Program epic: #3812
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Implemented
- Priority: P1

## Objective

As a maintainer, I need shared shell substrate and data-driven dispatcher/wrapper wiring so that lane entrypoints stay deterministic without boilerplate explosion.

## Problem Statement

Phase 0-2 reduction requires systematic removal of duplicated shell boilerplate and static dispatch mappings.

## Scope

In scope:
- Issue-defined implementation slice and deterministic behavior contracts.
- Conformance and regression checks mapped to acceptance criteria.
- Shell-surface governance markers where script/workflow surface changes.

Out of scope:
- Unrelated runtime/product features.
- Non-deterministic policy behavior.

## Acceptance Criteria

- AC-1: Shared shell primitives replace duplicated helper implementations in targeted script families.
- AC-2: Dispatcher resolves manifests/phases via manifest metadata or generated index, not hardcoded case blocks.
- AC-3: Tiny exec wrappers are replaced with symlink/registry-driven entrypoints where safe.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
