# Spec: Issue 6212 - Cache Logging Config Instead of Per-Emission Env Reads

- Issue: #6212
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P2
- Area: backend

## Problem Statement

`emit_log_event` resolves logging config from environment on every emitted log
line. R59 flagged this as avoidable per-event overhead.

## Scope

In scope:
1. Add cached log-config resolution for runtime log emission.
2. Preserve current invalid-config fail-closed behavior.
3. Add regressions proving resolver invocation happens once per cache and
   failures are cached deterministically.

Out of scope:
1. Dynamic runtime log-level reload from environment.
2. Changes to log line format/schema.

## Acceptance Criteria

### AC-1 Cache-Based Resolution
Given repeated log config resolutions for the same runtime process,
When log config is requested multiple times,
Then resolver is invoked once and cached result is reused.

### AC-2 Error Caching Is Fail-Closed
Given an invalid log config on first resolution,
When resolution is retried,
Then cached failure is returned deterministically without re-resolving.

### AC-3 Log Rendering Behavior Unchanged
Given existing logging behavior tests,
When cache is introduced,
Then log format/level behavior remains unchanged.

## Conformance Cases

- C-01 (AC-1, Unit): `tests::regression_issue_6212_cached_log_config_resolver_invoked_once`
- C-02 (AC-2, Unit): `tests::regression_issue_6212_cached_log_config_caches_fail_closed_error`
- C-03 (AC-3, Unit): `tests::regression_issue_6212_cached_log_config_resolver_invoked_once` (config reuse returns same value)
