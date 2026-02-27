# Spec: Issue 6206 - Unify Task Lifecycle Transition Mapping

- Issue: #6206
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: backend

## Problem Statement

`task_lifecycle.rs` encodes lifecycle transitions twice:
`next_state(from, transition)` and `transition_between(from, to)`.
R59 flagged divergence risk if one table changes without the other.

## Scope

In scope:
1. Define one canonical transition table for task lifecycle edges.
2. Route both forward and reverse lookup helpers through this table.
3. Add regression tests to lock one-source-of-truth behavior.

Out of scope:
1. Changing allowed task lifecycle transitions.
2. Modifying task state or transition enums.

## Acceptance Criteria

### AC-1 Single Transition Source
Given lifecycle transition helpers,
When mapping forward or reverse edges,
Then both helpers resolve via one canonical table.

### AC-2 Forward/Reverse Consistency
Given every canonical transition edge,
When looking up forward and reverse mappings,
Then reverse lookup returns the originating transition.

### AC-3 Existing Behavior Preserved
Given existing lifecycle tests,
When unified table is introduced,
Then previous lifecycle semantics remain unchanged.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::regression_issue_6206_lifecycle_lookups_share_single_transition_table`
- C-02 (AC-2, Unit): `tests::regression_issue_6206_lifecycle_forward_and_reverse_lookup_are_consistent`
- C-03 (AC-3, Unit): `tests::restore_replays_valid_history`
