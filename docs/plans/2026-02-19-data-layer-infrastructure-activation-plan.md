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
- Task (completed): `#5263` — implement Phase-2 envelope+blind-index operational pipeline and persist blind indexes in adapter inserts
- Task (completed): `#5265` — implement M1 batch scheduler thresholds and merkle-batch persistence execution path
- Task (completed): `#5267` — implement M1 anchoring orchestrator tick and persistence-plan projection
- Task (completed): `#5269` — implement M1 anchoring follow-up retry and confirmation policy projection
- Task (completed): `#5271` — implement M1 finality-observation reconciliation projection
- Task (completed): `#5273` — implement M5 pgvector adapter projection and fail-closed extension contracts
- Task (completed): `#5275` — implement M6 AGE adapter projection and fail-closed graph-extension contracts
- Task (completed): `#5277` — implement M7 Timescale adapter projection and fail-closed telemetry-extension contracts
- Task (completed): `#5279` — implement M9 realtime gateway bridge projection and fail-closed presence-scope contracts
- Task (completed): `#5281` — integrate service-api websocket presence route with M9 gateway bridge contracts
- Task (completed): `#5283` — validate realtime backpressure guardrails and finalize presence gateway ops docs
- Task (completed): `#5285` — start Phase-6 retention-to-archival gate execution contracts
- Task (completed): `#5287` — add deterministic archival failure-retry policy contracts for Phase-6
- Task (completed): `#5289` — add Phase-6 retention+archival execution tick orchestration contracts
- Task (completed): `#5291` — add Phase-6 execution-tick budget guardrail contracts
- Task (completed): `#5293` — add Phase-6 scheduler-cycle trigger and guarded execution contracts
- Task (completed): `#5295` — add stateful Phase-6 scheduler runtime checkpoint contracts
- Task (completed): `#5297` — add Phase-6 runtime evidence bundle projection contracts
- Task (completed): `#5299` — wire Phase-6 retention scheduler runtime into kamn-node daemon tick
- Task (current wave): `#5301` — add deterministic convergence evidence projection and promotion markers

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
- `#5263`: merged to main (Phase-2 operational envelope+blind-index pipeline + adapter blind-index persistence).
- `#5265`: merged to main (Phase-3 scheduler trigger + merkle batch lifecycle persistence).
- `#5267`: merged to main (Phase-3 orchestrator tick + persistence-plan projection).
- `#5269`: merged to main (Phase-3 deterministic follow-up retry/confirmation policy projection).
- `#5271`: merged to main (Phase-3 finality-observation reconciliation projection).
- `#5273`: merged to main (Phase-4 pgvector adapter projection contracts).
- `#5275`: merged to main (Phase-4 AGE adapter projection contracts).
- `#5277`: merged to main (Phase-4 Timescale adapter projection contracts).
- `#5279`: merged to main (Phase-5 realtime gateway bridge projection contracts).
- `#5281`: merged to main (Phase-5 websocket presence-route integration with gateway bridge contracts).
- `#5283`: merged to main (Phase-5 guardrail validation + realtime ops-doc closure).
- `#5285`: merged to main (Phase-6 retention-to-archival gate execution kickoff).
- `#5287`: merged to main (Phase-6 archival failure-retry policy execution contracts).
- `#5289`: merged to main (Phase-6 retention+archival execution tick orchestration contracts).
- `#5291`: merged to main (Phase-6 execution-tick budget guardrail contracts).
- `#5293`: merged to main (Phase-6 scheduler-cycle trigger and guarded execution contracts).
- `#5295`: merged to main (stateful Phase-6 scheduler runtime checkpoint contracts).
- `#5297`: merged to main (Phase-6 runtime evidence bundle projection contracts).
- `#5299`: merged to main (Phase-6 daemon runtime integration wiring).
- `#5301`: in progress in this wave (cross-cutting convergence evidence projection markers).
- Remaining stories: planned, blocked on phase order dependencies.

## Shell-Surface Guardrail
- This plan adds no new shell/python/workflow/template surface.
- Expected shell markers for `#5255`:
  - `shell_loc_delta_actual: 0`
  - `shell_surface_ratio_target_status: improved`
