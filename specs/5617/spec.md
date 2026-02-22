# Issue #5617 Spec - Fix integration_config Flag Mapping in Run Output

- Status: Implemented
- Issue: #5617
- Parent: #5611
- Milestone: R52 E2E Live Runtime Integration Hardening

## Problem Statement
`execute_run_contract` currently serializes `integration_config.agent_binary_required` and `integration_config.external_execution_enabled` with swapped values. This makes run output misleading and violates deterministic contract semantics.

## Scope
In scope:
- Add RED tests asserting correct flag mapping for `sdk-direct` and `mcp-tau` modes.
- Fix serialization mapping in `execute_run_contract`.
- Add docs artifact and milestone marker updates.

Out of scope:
- Runtime spawn behavior changes.
- Protocol/wire-format changes beyond correcting current field mapping.

## Acceptance Criteria
- AC-1: `integration_config.external_execution_enabled` equals external request flag.
- AC-2: `integration_config.agent_binary_required` equals mode requirement (`mcp-*` true, non-mcp false).
- AC-3: RED->GREEN tests validate mapping in sdk and mcp modes.
- AC-4: docs/milestone markers are coherent.
- AC-5: required quality gates pass.

## Conformance Cases
- C-01 (AC-1): sdk-direct with external disabled maps `external_execution_enabled=false`.
- C-02 (AC-1): sdk-direct with external enabled maps `external_execution_enabled=true`.
- C-03 (AC-2): sdk-direct maps `agent_binary_required=false`.
- C-04 (AC-2): mcp-tau maps `agent_binary_required=true`.
- C-05 (AC-3): RED mapping tests fail before implementation.
- C-06 (AC-3): GREEN mapping tests pass after implementation.
- C-07 (AC-4): docs marker artifact exists and milestone index references #5617.
- C-08 (AC-5): `cargo fmt --check` passes.
- C-09 (AC-5): `cargo clippy -p kamn-e2e-harness -- -D warnings` passes.
- C-10 (AC-5): `cargo test -p kamn-e2e-harness` passes.
- C-11 (AC-5): `cargo test -p kamn-agent-lib` passes.
- C-12 (AC-5): `cargo test -p kamn-mcp-server -p kamn-cli` passes.

## Success Metrics / Observable Signals
- Run output integration configuration flags are semantically correct and deterministic.
- Flag semantics remain stable under both sdk and mcp external execution paths.
