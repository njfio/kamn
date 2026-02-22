# Issue #5578 Plan - PRD Phase-4h Live Runtime Binary Config Contracts

## Approach
1. Add RED conformance tests for parser acceptance/rejection and `integration_config` output markers.
2. Extend `RunCommandConfig` with runtime binary fields and parse new run flags.
3. Add mode-aware parser validation for MCP modes.
4. Emit deterministic `integration_config` in run output.
5. Add phase-4h docs markers and milestone progression update.
6. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase4h_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase4h-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: parser contract breakage for existing tests and call sites.
  - Mitigation: update all `RunCommandConfig` test fixtures in one slice and keep required fields deterministic.
- Risk: mode validation drift across parse and run paths.
  - Mitigation: enforce MCP requirement at parse-time and lock via conformance tests.

## Interfaces / Contracts
- New run flags:
  - `--kolme-binary <path>` (required)
  - `--agent-binary <path>` (MCP-required)
- New run output object:
  - `integration_config.kolme_binary`
  - `integration_config.agent_binary`
  - `integration_config.agent_binary_required`

## ADR
- Not required for deterministic contract extension.
