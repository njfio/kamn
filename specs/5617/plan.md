# Issue #5617 Plan - Fix integration_config Flag Mapping in Run Output

## Approach
1. Add RED tests for integration_config flag mapping under sdk/mcp + external on/off paths.
2. Fix mapping in `execute_run_contract` output formatting.
3. Add docs artifact and milestone index update.
4. Run required quality gates.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/r52_integration_config_mapping_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-r52-integration-config-mapping-fix.md` (new)
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: changing serialized values may break existing downstream expectations.
  - Mitigation: assert corrected deterministic mapping with explicit tests and retain field names/shape.
- Risk: regression in nearby phase-6 contract outputs.
  - Mitigation: run full harness and cross-crate regression suite.

## Interfaces / Contracts
- `integration_config.agent_binary_required` must map mode requirement.
- `integration_config.external_execution_enabled` must map external request flag.

## ADR
- Not required (bug fix within existing contract shape).
