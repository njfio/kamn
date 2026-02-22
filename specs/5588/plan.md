# Issue #5588 Plan - PRD Phase-5c Spawn Timeline Contracts

## Approach
1. Add RED tests for required spawn_timeline keys and canonical ordering values.
2. Implement deterministic `spawn_timeline` object in run output.
3. Add phase-5c docs marker artifact and milestone progression update.
4. Run quality gates and regressions.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `crates/kamn-e2e-harness/tests/phase5c_docs_contract.rs` (new)
- `docs/research/e2e-live-testing-prd-phase5c-gap-analysis.md` (new)
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: output-schema drift for downstream tooling.
  - Mitigation: enforce explicit key/value assertions for all timeline markers.
- Risk: sequencing ambiguity.
  - Mitigation: lock single canonical step numbering in tests.

## Interfaces / Contracts
- `spawn_timeline.postgres_start`
- `spawn_timeline.kolme_start`
- `spawn_timeline.kamn_nodes_start`
- `spawn_timeline.agent_deploy_start`

## ADR
- Not required for deterministic contract extension.
