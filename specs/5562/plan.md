# Issue #5562 Plan - PRD Phase-3 kamn-e2e-harness Scaffold and Core Scenario Contracts

## Approach
1. Add RED conformance tests for harness structure, execution modes, scenario inventory, and manifest schema.
2. Scaffold `kamn-e2e-harness` modules:
   - `main.rs`, `infrastructure.rs`, `kolme_devnet.rs`, `identity.rs`
   - `drivers/{mod,sdk_direct,cli_scripted,mcp_agent}.rs`
   - `scenarios/{mod,s01_discovery,s02_message,s03_group,s04_task,s05_escrow,s06_kolme_verify,s08_crash_recovery}.rs`
   - `evidence.rs`, `verify.rs`
3. Implement deterministic mode/scenario registries and evidence manifest/verification structures.
4. Add docs/research phase-3 gap status markers.
5. Run fmt/clippy/targeted tests and record GREEN evidence.

## Affected Modules
- `Cargo.toml` (workspace member)
- `crates/kamn-e2e-harness/Cargo.toml`
- `crates/kamn-e2e-harness/src/**`
- `crates/kamn-e2e-harness/tests/**`
- `docs/research/e2e-live-testing-prd-phase3-gap-analysis.md`
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: scope creep into real infra orchestration.
  - Mitigation: keep this issue to deterministic scaffolds and conformance contracts only.
- Risk: mismatch between PRD scenario naming and code identifiers.
  - Mitigation: use canonical scenario IDs (`S-xx`) and explicit mapping tests.
- Risk: evidence schema drift.
  - Mitigation: centralize manifest schema version constant + verification tests.

## Interfaces / Contracts
- Mode registry contract:
  - `sdk-direct`, `cli-scripted`, `mcp-tau`, `mcp-any`
- Scenario contract IDs:
  - `S-01`, `S-02`, `S-03`, `S-04`, `S-05`, `S-06`, `S-08`
- Evidence manifest schema marker:
  - `kamn.e2e.evidence-manifest.v3`

## ADR
- Not required (scaffold implementation with deterministic contracts).
