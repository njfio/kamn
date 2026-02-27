# Spec: Issue 6201 - Reduce E2E Driver Duplication Surface

- Issue: #6201
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P1
- Area: qa

## Problem Statement

`kamn-e2e-harness` driver implementations (`sdk_direct.rs`, `cli_scripted.rs`,
`mcp_agent.rs`) duplicate a large set of low-level helper logic (env parsing,
live-scenario gating, latency budget checks, replay marker checks). This
increases maintenance cost and creates behavior skew risk.

## Scope

In scope:
1. Extract shared deterministic helper functions used across multiple drivers.
2. Update all three drivers to consume shared helpers.
3. Add unit coverage for shared helper behavior used by live probes.

Out of scope:
1. Full scenario probe unification into a single generic driver.
2. Renaming scenario payload constants.
3. Wire-protocol behavior changes.

## Acceptance Criteria

### AC-1 Shared Helper Module Introduced
Given duplicated helper logic across driver files,
When driver modules compile,
Then shared helper functions are provided by a common driver helper module.

### AC-2 Driver-Level Duplicates Reduced
Given helper categories (bool/env parsing, live-scenario gates, S07 marker validation, S15 percentile/latency validation),
When scanning driver files,
Then local duplicate helper implementations are removed in favor of shared calls.

### AC-3 Regression Coverage Added
Given shared helper behavior,
When unit tests run,
Then helper contracts for boolean parsing, live-scenario membership, and percentile math pass.

## Conformance Cases

- C-01 (AC-1, Unit): `drivers::shared::tests::spec_c01_live_scenario_gate_accepts_expected_ids`
- C-02 (AC-2, Unit): `drivers::shared::tests::spec_c02_validate_replay_reason_marker_requires_marker`
- C-03 (AC-3, Unit): `drivers::shared::tests::spec_c03_percentile_index_is_monotonic_and_bounded`
