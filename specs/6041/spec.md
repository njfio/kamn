# Spec: Issue #6041 - Cache node log config to avoid per-log env lookups

- Issue: #6041
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #5973

## Problem Statement
`kamn-node` logging currently resolves `KAMN_NODE_LOG_LEVEL` and `KAMN_NODE_LOG_FORMAT` via `env::var` for each emitted log event. This introduces avoidable per-call overhead in hot paths and repeats config parsing work.

## Scope
In scope:
- Add cached log config resolution (first-use or startup-resolved) for node logging.
- Preserve existing log level and output format behavior.
- Add deterministic tests for cache reuse and test reset semantics.

Out of scope:
- Logging subsystem replacement.
- Changes to rendered log schema/fields.

## Risk Level
`low`

## Acceptance Criteria
- AC-1: `emit_log_event` does not call environment resolution on every call after initial cache population.
- AC-2: Text/JSON render behavior and level filtering remain unchanged.
- AC-3: Tests can reset cache deterministically to isolate environment-dependent assertions.

## Conformance Cases
- C-01 (Conformance, AC-1): Multiple log emissions under same process observe shared cached config and stable output mode.
- C-02 (Unit, AC-2): Cached config still enforces level filtering and output formatting parity.
- C-03 (Regression, AC-3): Test-only reset path clears cache so subsequent env changes are observed deterministically.

## Success Metrics / Observable Signals
- New logging cache tests pass in `kamn-node`.
- Existing logging tests remain green without behavior regressions.
