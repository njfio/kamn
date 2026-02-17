# Spec — Issue #4871

- Title: Task: enforce shell-surface DoR/DoD contracts across AGENTS, templates, and PR docs
- Parent: - Parent story: #4863
- Milestone: R27.43 Shell LOC maintainability and shell-to-Rust ratio sustainment governance
- Status: Reviewed
- Priority: P1

## Objective

Require shell delta + mitigation declarations and docs-contract tests across planning and review workflows.

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

- AC-1: Issue and PR templates fail contract tests when shell governance markers are missing.
- AC-2: AGENTS and contributing docs codify shell-surface DoR/DoD expectations.
- AC-3: Future script growth requires linked mitigation issue or explicit bounded exception path.

## Conformance Cases

- C-01: verify AC-1 with deterministic pass/fail evidence.
- C-02: verify AC-2 with deterministic pass/fail evidence.
- C-03: verify AC-3 with deterministic pass/fail evidence.

## Success Metrics / Signals

- Required tests pass with deterministic markers and stable reason-taxonomy outputs.
- Script-surface impact is measurable (reduction or bounded-containment) for this issue scope.
