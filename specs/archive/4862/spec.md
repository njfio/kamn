# Spec — Issue #4862

- Title: Story: consolidate policy checkers and generate manifests from registry source (phases 6-7)
- Parent: - Program epic: #3812
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Implemented
- Priority: P1

## Objective

As a maintainer, I need declarative policy-checker and generated lane registry architecture so that shell wrappers and manual manifest wiring stop growing linearly.

## Problem Statement

Phase 6-7 requires policy checker consolidation and generated manifests to prevent manual script proliferation.

## Scope

In scope:
- Issue-defined implementation slice and deterministic behavior contracts.
- Conformance and regression checks mapped to acceptance criteria.
- Shell-surface governance markers where script/workflow surface changes.

Out of scope:
- Unrelated runtime/product features.
- Non-deterministic policy behavior.

## Acceptance Criteria

- AC-1: Eligible policy checker scripts migrate to declarative framework with compatibility markers.
- AC-2: Lane registry becomes source of truth for manifest and wrapper generation.
- AC-3: Drift tests fail closed when generated artifacts diverge from registry source.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
