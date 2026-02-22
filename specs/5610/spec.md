# Issue #5610 Spec - External Execution Preflight Executable Diagnostics

- Status: Implemented
- Issue: #5610
- Parent: #5611
- Milestone: R52 E2E Live Runtime Integration Hardening

## Problem Statement
`kamn-e2e-harness` external preflight currently validates only binary path existence. Misconfigured non-executable binaries pass this gate and fail later with less-actionable diagnostics.

## Scope
In scope:
- Validate `kolme_binary` is executable when external execution is enabled.
- Validate MCP-mode `agent_binary` is executable when external execution is enabled.
- Return deterministic preflight errors for non-executable binaries.
- Add RED->GREEN tests proving failures and pass path behavior.
- Update R52 milestone index and docs artifact.

Out of scope:
- Long-lived daemon spawn orchestration.
- Protocol/wire-format changes.

## Acceptance Criteria
- AC-1: preflight fails deterministically for non-executable `kolme_binary`.
- AC-2: preflight fails deterministically for non-executable MCP `agent_binary`.
- AC-3: executable binaries continue to pass preflight and run contract output renders.
- AC-4: RED->GREEN tests validate behavior.
- AC-5: required quality gates pass.

## Conformance Cases
- C-01 (AC-1): `sdk-direct` external execution with non-executable `kolme_binary` returns deterministic error.
- C-02 (AC-2): `mcp-tau` external execution with non-executable `agent_binary` returns deterministic error.
- C-03 (AC-3): executable temp binaries pass preflight in `sdk-direct` and `mcp-tau` paths.
- C-04 (AC-4): RED tests fail before implementation.
- C-05 (AC-4): GREEN tests pass after implementation.
- C-06 (AC-4): docs/milestone markers pass.
- C-07 (AC-5): `cargo fmt --check` passes.
- C-08 (AC-5): `cargo clippy -p kamn-e2e-harness -- -D warnings` passes.
- C-09 (AC-5): `cargo test -p kamn-e2e-harness` passes.
- C-10 (AC-5): `cargo test -p kamn-agent-lib` passes.
- C-11 (AC-5): `cargo test -p kamn-mcp-server -p kamn-cli` passes.

## Success Metrics / Observable Signals
- External execution preflight catches executability defects before runtime launch.
- Error messages are deterministic and mode-specific.
