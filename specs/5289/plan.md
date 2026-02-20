# Issue #5289 Plan

## Objective
Implement a deterministic Phase-6 orchestration tick that composes retention/shredding (M8) and partition projection/archival (M10) into one explicit contract boundary with stable reason markers.

## Approach
1. Add a new orchestration contract type set in `kamn-core`:
   - execution request
   - execution report
   - orchestration error + reason markers
2. Implement one orchestration function that:
   - queries M8 retention-due candidates for owner scope
   - executes M8 crypto-shred for each due candidate
   - applies M10 shred-completeness projection for provided partition message maps
   - executes M10 due archival selection
3. Ensure deterministic ordering:
   - shredded message IDs sorted
   - projection reports sorted by partition month
   - archival entries in deterministic partition order
4. Add conformance tests (`C-01`..`C-06`) and update tracker docs for current wave.

## Affected Modules
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`

## Risks and Mitigations
- Risk: mixed M8/M10 error surfaces may produce non-deterministic operator signals.
  - Mitigation: add explicit orchestration error variants with stable reason codes and deterministic mapping.
- Risk: map/set iteration can produce unstable ordering.
  - Mitigation: use `BTreeMap`/sorted vectors for externally visible report fields.
- Risk: legal-hold paths could partially mutate state before failing.
  - Mitigation: execute shredding in deterministic sequence and fail immediately with explicit error markers.

## Interfaces / Contracts
- New public orchestration API exported from `kamn-core` with stable reason-code constants.
- Existing M8 and M10 registry contracts are composed without wire/protocol/schema changes.

## ADR
- Not required (no new dependency, protocol, or architecture boundary introduced).
