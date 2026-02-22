# Issue #5613 Spec - External Preflight Rejects Non-File Binary Paths

- Status: Implemented
- Issue: #5613
- Parent: #5611
- Milestone: R52 E2E Live Runtime Integration Hardening

## Problem Statement
External preflight currently validates path existence + executability, but an existing non-file path (for example directory) can still satisfy executability checks and produce misleading readiness results.

## Scope
In scope:
- Reject existing but non-regular-file `kolme_binary` paths in external mode.
- Reject existing but non-regular-file MCP `agent_binary` paths in external mode.
- Preserve executable-bit checks and deterministic diagnostics from #5610.
- Add RED->GREEN tests and docs/milestone updates.

Out of scope:
- Runtime spawn orchestration.
- Protocol/wire-format changes.

## Acceptance Criteria
- AC-1: preflight fails deterministically for non-file `kolme_binary` path.
- AC-2: preflight fails deterministically for non-file MCP `agent_binary` path.
- AC-3: regular-file executable binaries continue to pass preflight.
- AC-4: RED->GREEN tests validate behavior.
- AC-5: required quality gates pass.

## Conformance Cases
- C-01 (AC-1): directory path provided as `kolme_binary` returns deterministic non-file error.
- C-02 (AC-2): directory path provided as MCP `agent_binary` returns deterministic non-file error.
- C-03 (AC-3): executable regular files pass preflight in `sdk-direct` and `mcp-tau`.
- C-04 (AC-4): RED tests fail before implementation.
- C-05 (AC-4): GREEN tests pass after implementation.
- C-06 (AC-4): docs/milestone contract tests pass.
- C-07 (AC-5): `cargo fmt --check` passes.
- C-08 (AC-5): `cargo clippy -p kamn-e2e-harness -- -D warnings` passes.
- C-09 (AC-5): `cargo test -p kamn-e2e-harness` passes.
- C-10 (AC-5): `cargo test -p kamn-agent-lib` passes.
- C-11 (AC-5): `cargo test -p kamn-mcp-server -p kamn-cli` passes.

## Success Metrics / Observable Signals
- External preflight rejects non-file binary paths with deterministic diagnostics.
- Runtime readiness signals are safer and more actionable.
