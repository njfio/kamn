# Issue #5279 Plan

- Issue: #5279
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Add a dedicated M9 gateway bridge module in `kamn-core` that maps M9 dispatch/presence contract outcomes into deterministic transport envelopes.
2. Support explicit gateway transport profile normalization with fail-closed rejection for unsupported profiles.
3. Project presence connect/query flows through `DataLayerM9RealtimeDeliveryRegistry` and surface scope/visibility denials without lossy mapping.
4. Add RED tests for deterministic dispatch/presence projection and fail-closed branches.
5. Export the new gateway bridge APIs through `lib.rs`.
6. Run scoped verification (`fmt`, strict `clippy`, targeted bridge tests).

## Affected Areas
- `crates/kamn-core/src/data_layer_m9_gateway_bridge.rs` (new)
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_m9_gateway_bridge.rs` (new)
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `specs/5279/{spec.md,plan.md,tasks.md}`

## Risks and Mitigations
- Risk: bridge layer could drift from M9 reason taxonomy.
  - Mitigation: preserve source reason markers in bridge errors and assert exact reason-code values in regression tests.
- Risk: transport profile handling may overfit one protocol.
  - Mitigation: normalize two supported profiles (`websocket`, `sse`) and fail closed for unknown values.
- Risk: scope/visibility behavior could be obscured by wrapper errors.
  - Mitigation: explicitly map and expose M9 owner-scope and visibility denials in the bridge error taxonomy.

## Interfaces / Contracts
- New gateway bridge projection functions and envelope/error structs in `kamn-core`.
- No live network I/O in this issue; this is deterministic adapter-boundary projection.
- M9 contract module remains source-of-truth decision logic.

## ADR
Not required; this is an incremental Phase-5 adapter-contract slice under existing milestone architecture.
