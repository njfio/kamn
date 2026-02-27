# Spec: Issue 6211 - Replace Censorship Ratio f64 Arithmetic with Integer Math

- Issue: #6211
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: backend

## Problem Statement

`WatchdogNode::observe_gossip` computed `observed_ratio_pct` using `f64`
division/floor conversion. R59 flagged this as avoidable nondeterminism for
consensus-adjacent runtime logic.

## Scope

In scope:
1. Replace float-based ratio computation with integer floor division.
2. Keep behavior equivalent to previous floor semantics.
3. Add regression tests for ratio floor behavior and large-value safety.

Out of scope:
1. Changing watchdog alert thresholds.
2. Altering watchdog alert payload schema.

## Acceptance Criteria

### AC-1 Integer Ratio Computation
Given gossip observation processing,
When computing `observed_ratio_pct`,
Then the value is derived using integer arithmetic only.

### AC-2 Floor Semantics Preserved
Given delivered/expected ratios with fractional percentages,
When watchdog computes the ratio,
Then results match previous floor behavior.

### AC-3 Large Inputs Stay Safe
Given large valid recipient counts,
When watchdog computes ratio percent,
Then arithmetic remains bounded and deterministic.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::regression_issue_6211_delivery_ratio_pct_uses_integer_floor_math`
- C-02 (AC-2, Unit): `tests::regression_issue_6211_delivery_ratio_pct_uses_integer_floor_math`
- C-03 (AC-3, Unit): `tests::regression_issue_6211_delivery_ratio_pct_handles_large_values_without_overflow`
