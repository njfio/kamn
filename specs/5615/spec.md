# Issue #5615 Spec - External Preflight Requires Absolute Binary Paths

- Status: Implemented
- Issue: #5615
- Parent: #5611
- Milestone: R52 E2E Live Runtime Integration Hardening

## Problem Statement
External preflight currently permits relative binary paths. Relative-path execution is environment-dependent and can hide operator misconfiguration.

## Scope
In scope:
- Reject relative `kolme_binary` path when external execution is enabled.
- Reject relative MCP `agent_binary` path when external execution is enabled.
- Preserve existing checks for existence, regular-file, and executability.
- Add RED->GREEN tests and docs/milestone updates.

Out of scope:
- Runtime spawn orchestration.
- Protocol/wire-format changes.

## Acceptance Criteria
- AC-1: preflight fails deterministically for relative `kolme_binary` path.
- AC-2: preflight fails deterministically for relative MCP `agent_binary` path.
- AC-3: absolute regular-file executable binaries continue to pass preflight.
- AC-4: RED->GREEN tests validate behavior.
- AC-5: required quality gates pass.

## Conformance Cases
- C-01 (AC-1): relative `kolme_binary` path returns deterministic absolute-path error.
- C-02 (AC-2): relative MCP `agent_binary` path returns deterministic absolute-path error.
- C-03 (AC-3): absolute executable files pass preflight.
- C-04 (AC-4): RED tests fail before implementation.
- C-05 (AC-4): GREEN tests pass after implementation.
- C-06 (AC-4): docs/milestone contract tests pass.
- C-07 (AC-5): `cargo fmt --check` passes.
- C-08 (AC-5): `cargo clippy -p kamn-e2e-harness -- -D warnings` passes.
- C-09 (AC-5): `cargo test -p kamn-e2e-harness` passes.
- C-10 (AC-5): `cargo test -p kamn-agent-lib` passes.
- C-11 (AC-5): `cargo test -p kamn-mcp-server -p kamn-cli` passes.

## Success Metrics / Observable Signals
- External preflight rejects relative path ambiguity deterministically.
- Operators receive actionable absolute-path diagnostics before runtime launch.
