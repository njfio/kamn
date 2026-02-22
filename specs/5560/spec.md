# Issue #5560 Spec - PRD Phase-2 kamn-mcp-server and kamn-cli Foundation Implementation

- Status: Implemented
- Issue: #5560
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
PRD phase-2 requires `kamn-mcp-server` and `kamn-cli` wrapper binaries over `kamn-agent-lib`, but both crates and their required source layouts are missing.

## Scope
In scope:
- Add workspace crates `crates/kamn-mcp-server` and `crates/kamn-cli` with PRD section-13 phase-2 structure.
- Implement deterministic command/tool scaffolds mapped to phase-1 `kamn-agent-lib` APIs.
- Add conformance tests for MCP tool inventory/schema and CLI subcommand surface.
- Add phase-2 gap/status research artifact.

Out of scope:
- CI workflow edits.
- External agent runtime orchestration.
- E2E harness crate delivery.

## Acceptance Criteria
- AC-1: Workspace contains `kamn-mcp-server` and `kamn-cli` crates with required phase-2 files and successful compilation.
- AC-2: MCP server exposes deterministic 12-tool registry with schema descriptors.
- AC-3: CLI exposes deterministic 12-subcommand surface with JSON/text format controls and env overrides.
- AC-4: RED->GREEN tests validate phase-2 structure and command/tool contracts.
- AC-5: Docs/research phase-2 gap/status markers are present.
- AC-6: Quality gates pass (`cargo fmt --check`, `cargo clippy -p kamn-mcp-server -- -D warnings`, `cargo clippy -p kamn-cli -- -D warnings`, targeted tests).

## Conformance Cases
- C-01 (AC-1): phase-2 required crate/file paths exist.
- C-02 (AC-1): workspace compiles `kamn-mcp-server` and `kamn-cli`.
- C-03 (AC-2): MCP tool registry contains exactly 12 required tools.
- C-04 (AC-2): each MCP tool has deterministic input/output schema markers.
- C-05 (AC-3): CLI parser accepts each required subcommand.
- C-06 (AC-3): CLI format/env behavior markers are deterministic.
- C-07 (AC-4): RED failures then GREEN pass for structure/tool/command tests.
- C-08 (AC-5): phase-2 docs/research markers present and internally coherent.
- C-09 (AC-6): fmt/clippy/tests green.

## Success Metrics / Observable Signals
- Phase-2 crates are present and testable in workspace.
- MCP and CLI surfaces are stable enough for phase-3 harness drivers.
- Phase-2 gap baseline transitions from missing to implemented.
