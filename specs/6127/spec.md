# Spec: Issue #6127 - Consolidate task lifecycle transition logic into a single table

- Issue: #6127
- Status: Accepted
- Type: task
- Priority: P2
- Area: backend
- Milestone: `specs/milestones/r68-r59-swarm-remediation-and-full-gap-closure/index.md`
- Last Updated: 2026-02-27
- Parent: #6101

## Problem Statement
`task_lifecycle.rs` duplicates transition rules across `next_state` and `transition_between`. This creates silent divergence risk: one map can change without the other, causing restore/forward-transition mismatches.

## Scope
In scope:
- Represent lifecycle transitions in one canonical table.
- Make both transition lookup directions read from that table.
- Add regression coverage that enforces bidirectional consistency and uniqueness.

Out of scope:
- Expanding lifecycle states or transitions.
- Changing terminal-state policy semantics.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: `next_state` and `transition_between` derive from one transition table.
- AC-2: Regression test asserts one-to-one mapping consistency for both lookup directions.
- AC-3: Existing lifecycle behavior tests remain green.

## Conformance Cases
- C-01 (Unit, AC-1): `next_state` returns values sourced from canonical transition table.
- C-02 (Unit/Regression, AC-2): `regression_transition_edge_table_is_bidirectionally_consistent` passes.
- C-03 (Conformance, AC-3): `cargo test -p kamn-core task_lifecycle::tests:: -- --nocapture` passes.

## Success Metrics / Observable Signals
- Transition rules exist in one authoritative data structure.
- Any future duplicate/divergent transition entry fails deterministic regression coverage.
