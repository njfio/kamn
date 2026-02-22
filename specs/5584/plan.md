# Issue #5584 Plan - PRD Phase-5a Process Runtime Inventory Contracts

## Approach
1. Add RED tests for process_runtime marker presence and mode-aware agent runtime values.
2. Implement `process_runtime` composition in run output.
3. Add phase-5a docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase5a_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase5a-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: output-schema drift for downstream tooling.
  - Mitigation: add explicit conformance markers in tests.
- Risk: mode mapping inconsistency.
  - Mitigation: lock mode mapping with explicit assertions per mode.

## Interfaces / Contracts
- `process_runtime.kolme_runtime`
- `process_runtime.kamn_nodes_runtime`
- `process_runtime.agent_runtime`
- `process_runtime.spawn_strategy`

## ADR
- Not required for deterministic contract extension.
