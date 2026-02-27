# Spec: Issue 6210 - Bound Anti-Spam Seen-Message-ID Growth

- Issue: #6210
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: security

## Problem Statement

`AntiSpamEngine` tracked `seen_message_ids` in an unbounded `HashSet`, allowing
monotonic memory growth under long-running traffic.

## Scope

In scope:
1. Add deterministic bounded capacity for seen message IDs.
2. Evict oldest retained message IDs when capacity is exceeded.
3. Add regressions for eviction behavior and invalid zero-capacity config.

Out of scope:
1. Time-based eviction.
2. Cross-process persistence for anti-spam replay state.

## Acceptance Criteria

### AC-1 Capacity Bound Enforced
Given anti-spam config with `max_seen_message_ids=N`,
When more than `N` unique message IDs are evaluated,
Then engine evicts oldest IDs and retains at most `N`.

### AC-2 Eviction Is Deterministic
Given insertion order `msg-1`, `msg-2`, `msg-3` with `N=2`,
When evaluating duplicates afterward,
Then `msg-2` remains duplicate-rejected and `msg-1` is accepted after eviction.

### AC-3 Zero Capacity Rejected
Given anti-spam config with `max_seen_message_ids=0`,
When creating engine,
Then constructor fails closed with deterministic `InvalidConfig`.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::regression_issue_6210_seen_message_ids_evict_oldest_entry_at_capacity`
- C-02 (AC-2, Unit): `tests::regression_issue_6210_seen_message_ids_evict_oldest_entry_at_capacity`
- C-03 (AC-3, Unit): `tests::regression_issue_6210_config_rejects_zero_seen_message_capacity`

