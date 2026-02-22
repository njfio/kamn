# Issue #5564 Plan - PRD Phase-4a Scenario Matrix and Evidence Verifier Contract Completion

## Approach
1. Write RED conformance tests that require:
   - full scenario matrix (`S-01..S-15`)
   - PRD section-8.2 manifest fields
   - deterministic verifier report markers
2. Extend scenario modules and registry contracts:
   - add missing `S-07`, `S-09`, `S-10`, `S-11`, `S-12`, `S-13`, `S-14`, `S-15`
   - add full-matrix registry function
   - update default harness run plan to schedule full matrix
3. Extend `evidence.rs` model contracts:
   - typed infrastructure/scenario/summary marker structs
   - deterministic manifest builder with schema pin
4. Extend `verify.rs`:
   - deterministic report data structure
   - deterministic report renderer containing schema/proof/chain/content check markers
5. Update docs/research + milestone index markers.
6. Run fmt/clippy/targeted tests and record RED->GREEN evidence.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/scenarios/mod.rs`
- `crates/kamn-e2e-harness/src/scenarios/*.rs`
- `crates/kamn-e2e-harness/src/evidence.rs`
- `crates/kamn-e2e-harness/src/verify.rs`
- `crates/kamn-e2e-harness/tests/*.rs`
- `docs/research/e2e-live-testing-prd-phase4a-gap-analysis.md`
- `specs/milestones/r51-e2e-live-testing-prd-full-delivery/index.md`

## Risks and Mitigations
- Risk: introducing schema drift with ad-hoc field names.
  - Mitigation: hard-code PRD section-8.2 names in tests before implementation.
- Risk: run-plan behavior regression for prior phase tests.
  - Mitigation: update and rerun phase-3/phase-4 conformance tests together.
- Risk: non-deterministic report formatting.
  - Mitigation: keep report generation pure and field ordering fixed in renderer.

## Interfaces / Contracts
- Scenario registry contract:
  - IDs: `S-01`..`S-15`
  - deterministic name/priority mapping from PRD section 7.1
- Manifest schema contract:
  - `schema_version = kamn.e2e.evidence-manifest.v3`
  - infrastructure + per-scenario + top-level summary markers per PRD section 8.2
- Verifier report contract:
  - includes `schema_check`, `proof_check`, `chain_check`, `content_check` markers
  - deterministic output for same input

## ADR
- Not required (contract completion and deterministic model extension within existing architecture).
