# Issue #5287 Plan

## Objective
Model archival failure/retry behavior as deterministic M10 contracts so Phase-6 execution has explicit recoverable and terminal failure projections.

## Affected Modules
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/tests/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/lib.rs`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`

## Approach
1. Add failing conformance tests first for transient retry, backoff cap behavior, exhausted retries, permanent failure, and invalid-policy inputs.
2. Implement retry policy projection types/constants/functions in M10 with deterministic backoff math and stable reason codes.
3. Export new contracts through `lib.rs` and keep behavior side-effect free.
4. Run targeted verification (`fmt`, strict `clippy`, M10 test suite), then prepare PR evidence.

## Risks and Mitigations
- Risk: retry backoff arithmetic overflow or drift.
  - Mitigation: saturating math + capped exponent and explicit max-backoff clamp tests.
- Risk: ambiguous failure taxonomy between transient/permanent classes.
  - Mitigation: explicit failure-class enum and reason-code mapping tests.
- Risk: policy misconfiguration accepted silently.
  - Mitigation: fail-closed policy validation with deterministic error variants.

## Interfaces and Contracts
- Add deterministic projection function(s) and small supporting policy types only; no external I/O.
- Preserve existing M10 lifecycle registry behavior.
- Keep reason markers stable for docs/governance lane consumers.

## ADR
- No ADR required; this is a scoped contract extension without new dependencies or protocol changes.
