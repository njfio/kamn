# Spec: Issue #6122 - E2E Driver Shared Helper Deduplication

Status: Reviewed
Issue: #6122
Milestone: r68-r59-swarm-remediation-and-full-gap-closure

## Problem Statement
`kamn-e2e-harness` keeps duplicated helper logic across `sdk_direct.rs`, `cli_scripted.rs`, and `mcp_agent.rs` (env parsing, live-scenario gating, replay marker checks, percentile/budget math, and repeated validation helpers). The duplication increases maintenance risk and drifts fixes across drivers.

## Scope
In scope:
- Extract duplicated helper logic into a shared drivers utility module.
- Update all three driver implementations to use shared helpers without behavior changes.
- Preserve/adjust tests to prove shared helper behavior and driver conformance behavior remain stable.

Out of scope:
- Rewriting scenario probe flows.
- Changing public API behavior of harness drivers.
- Refactoring unrelated crates.

## Acceptance Criteria
- AC-1: Duplicated helper implementations for common env/live/replay/latency/content/bridge checks are centralized in one shared driver helper module.
- AC-2: `sdk_direct`, `cli_scripted`, and `mcp_agent` use shared helper functions instead of local duplicated implementations for the targeted helper set.
- AC-3: Existing regression/conformance behavior for live gating and helper validation remains intact under tests.

## Conformance Cases
- C-01 (AC-1): Shared helper module exists and exports the targeted helper set used by all three drivers.
- C-02 (AC-2): Each driver compiles and references shared helpers for duplicated logic (`env_var_or_default`, `env_var_or_else`, scenario/live gating, replay marker checks, percentile/budget checks, and content/bridge validators).
- C-03 (AC-3): `cargo test -p kamn-e2e-harness` passes; specific regression tests for live gating/replay/latency helper behavior remain green.

## Success Metrics
- `cargo test -p kamn-e2e-harness`
- Reduced duplicated helper definitions across the three driver files.
