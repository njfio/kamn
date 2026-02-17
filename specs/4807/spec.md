# Spec — Issue #4807

- Title: Story: build shared shell substrate and eliminate dispatch/wrapper duplication
- Parent: Parent epic: #4806
- Milestone: R27.42 Shell LOC reduction and script-to-Rust ratio inversion governance
- Status: Reviewed
- Priority: P1

## Objective

Execute phases 0-2 by centralizing shell primitives, removing dispatcher hardcoding, and replacing tiny exec wrappers with maintainable indirection.

## Problem Statement

Repeated ROOT_DIR/assert/usage boilerplate and hardcoded dispatcher/wrapper mapping creates high maintenance churn and duplicate bug surface.

## Scope

In scope:
- scripts/lib/common.sh foundation and migration
- manifest-driven dispatcher resolution
- registry-based wrapper elimination

Out of scope:
- domain-level policy semantics changes unrelated to dispatch surface
- non-shell runtime feature work

## Acceptance Criteria

- AC-1: Common shell library replaces duplicated primitives in a measurable first migration wave.
- AC-2: Dispatcher no longer requires hardcoded lane case maintenance for manifest resolution.
- AC-3: Tiny exec wrapper population is reduced through registry/symlink strategy with compatibility tests.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence and fail-closed reasons.
- C-02: verify AC-2 with deterministic pass/fail evidence and fail-closed reasons.
- C-03: verify AC-3 with deterministic pass/fail evidence and fail-closed reasons.

## Success Metrics / Signals

- Required tests for this scope pass and emit deterministic governance markers.
- Shell-surface reduction or containment impact is explicitly measurable for this scope.
