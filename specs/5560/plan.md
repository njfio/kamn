# Issue #5560 Plan - PRD Phase-2 kamn-mcp-server and kamn-cli Foundation Implementation

## Approach
1. Add RED conformance tests asserting phase-2 structure and required MCP/CLI surfaces.
2. Scaffold `kamn-mcp-server`:
   - `main.rs` argument parsing + deterministic JSON output mode.
   - `config.rs` identity/endpoint config validation.
   - `tools.rs` 12-tool registry with schema descriptors.
3. Scaffold `kamn-cli`:
   - `main.rs` parser/dispatch for 12 required subcommands.
   - `commands/*` module files per PRD structure.
   - deterministic `--format json|text` and `KAMN_*` env resolution.
4. Add phase-2 gap analysis research markers.
5. Run quality gates and targeted regression checks.

## Affected Modules
- `Cargo.toml` (workspace members)
- `crates/kamn-mcp-server/Cargo.toml`
- `crates/kamn-mcp-server/src/main.rs`
- `crates/kamn-mcp-server/src/config.rs`
- `crates/kamn-mcp-server/src/tools.rs`
- `crates/kamn-cli/Cargo.toml`
- `crates/kamn-cli/src/main.rs`
- `crates/kamn-cli/src/commands/*.rs`
- `docs/research/e2e-live-testing-prd-phase2-gap-analysis.md`
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: introducing parser dependencies that violate repo constraints.
  - Mitigation: start with std-only deterministic parsers; avoid new external deps.
- Risk: mismatch between PRD command/tool names and current service capabilities.
  - Mitigation: preserve required surface names while returning explicit unsupported errors where back-end route support is absent.
- Risk: phase-2 scaffolds drift from phase-1 handle contracts.
  - Mitigation: route all command/tool execution through `kamn-agent-lib` adapters.

## Interfaces / Contracts
- MCP tool registry contracts:
  - 12 exact tool names: register, send_message, create_channel, list_messages, query_message, create_task, accept_task, complete_task, fund_escrow, release_escrow, verify_proof, health.
  - deterministic JSON schema descriptor for each tool.
- CLI contracts:
  - 12 exact subcommands matching MCP tool names (kebab-case where required).
  - `--format json|text` output and `KAMN_ENDPOINT` env fallback.

## ADR
- Not required for additive phase-2 wrapper scaffolds.
