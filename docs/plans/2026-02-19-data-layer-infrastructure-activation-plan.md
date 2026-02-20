# 2026-02-19 Data Layer Infrastructure Activation Plan

## Source and Intent
- Source review: `docs/review/data-layer-roadmap.md`
- Milestone container: `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- Goal: close the contract-to-runtime gap by activating infrastructure phases for data-layer modules M0-M11.

## Execution Hierarchy
- Epic: `#5247` — activate data-layer infrastructure layer and close contract-to-runtime gap
- Story (Phase 1): `#5248` — build PostgreSQL persistence foundation and RLS contract bridge
- Story (Phase 2): `#5249` — implement envelope crypto and blind-index operational pipeline
- Story (Phase 3): `#5250` — wire Kolme anchoring client and merkle batch scheduler
- Story (Phase 4): `#5251` — deliver extension-backed intelligence adapters
- Story (Phase 5): `#5252` — implement realtime delivery gateway and owner-scoped presence service
- Story (Phase 6): `#5253` — automate retention crypto-shredding and partition archival execution
- Story (Cross-cutting): `#5254` — enforce contract-infrastructure convergence validation
- Task (completed): `#5255` — bootstrap PostgreSQL migration scaffolding and schema contract markers
- Task (completed): `#5257` — implement PostgreSQL repository bridge contracts and RLS session projection
- Task (completed): `#5259` — implement sqlx-backed PostgreSQL execution adapter and migration runner
- Task (completed): `#5261` — extend PostgreSQL execution adapter with RLS policy application and blind-index search execution
- Task (current wave): `#5263` — implement Phase-2 envelope+blind-index operational pipeline and persist blind indexes in adapter inserts

## Phase Plan
1. Phase 1 (PostgreSQL foundation): execute `#5248` with `#5255` bootstrap first, then repository + live RLS wiring.
2. Phase 2 (crypto + blind index): execute `#5249` after schema baseline is merged.
3. Phase 3 (Kolme anchoring runtime): execute `#5250` once persistence and batch tables are active.
4. Phase 4 (pgvector/AGE/Timescale): execute `#5251` on top of stable persistence/crypto contracts.
5. Phase 5 (realtime gateway): execute `#5252` after identity/session and persistence boundaries are live.
6. Phase 6 (compliance automation): execute `#5253` with M8/M10 lifecycle integration.
7. Cross-cutting convergence: run `#5254` throughout all phases as a promotion gate.

## Current Status
- `#5255`: merged to main (baseline migration scaffolding + marker tests).
- `#5257`: merged to main (deterministic SQL descriptor projection + M2 RLS statement projection).
- `#5259`: merged to main (live sqlx execution adapter + migration runner).
- `#5261`: merged to main (blind-index search execution + default RLS statement apply).
- `#5263`: implemented in this wave (Phase-2 operational envelope+blind-index pipeline + adapter blind-index persistence), pending merge.
- Remaining stories: planned, blocked on phase order dependencies.

## Shell-Surface Guardrail
- This plan adds no new shell/python/workflow/template surface.
- Expected shell markers for `#5255`:
  - `shell_loc_delta_actual: 0`
  - `shell_surface_ratio_target_status: improved`
