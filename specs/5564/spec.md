# Issue #5564 Spec - PRD Phase-4a Scenario Matrix and Evidence Verifier Contract Completion

- Status: Implemented
- Issue: #5564
- Parent: #5557
- Milestone: R51 E2E Live Testing PRD Full Delivery

## Problem Statement
Phase-3 delivered `kamn-e2e-harness` scaffolding, but the harness still lacks PRD-complete scenario inventory (`S-01` through `S-15`), manifest fields required by PRD section 8.2, and deterministic offline verification report output from PRD section 9.

## Scope
In scope:
- Extend scenario registry contracts to include all PRD matrix scenarios `S-01..S-15` with deterministic id/name/priority.
- Extend evidence manifest model with PRD section-8.2 fields:
  - run metadata
  - infrastructure markers
  - per-scenario summary fields
  - top-level summary counters
- Implement deterministic offline verification report output that includes schema/proof/chain/content check markers.
- Add RED->GREEN conformance tests for scenario inventory and manifest/report contracts.
- Add phase-4a docs/research status markers.

Out of scope:
- CI workflow changes (`.github/workflows/**`).
- Live infra process orchestration hardening.
- Full scenario execution logic for each `S-xx` case.

## Acceptance Criteria
- AC-1: Scenario registry exposes deterministic definitions for all PRD scenarios `S-01..S-15`.
- AC-2: Harness default run plan uses the full PRD scenario matrix (`S-01..S-15`).
- AC-3: Evidence manifest model includes PRD section-8.2 top-level, infrastructure, scenario, and summary markers.
- AC-4: Offline verifier emits deterministic report markers for schema/proof/chain/content checks.
- AC-5: RED->GREEN conformance tests validate scenario matrix completeness and manifest/report contracts.
- AC-6: Phase-4a docs/research status markers are present and coherent.
- AC-7: Quality gates pass (`cargo fmt --check`, `cargo clippy -p kamn-e2e-harness -- -D warnings`, targeted tests).

## Conformance Cases
- C-01 (AC-1): scenario inventory count is 15 and IDs are exactly `S-01..S-15`.
- C-02 (AC-1): scenario labels and priorities match PRD section 7.1 mapping.
- C-03 (AC-2): `build_core_run_plan` (default harness plan) schedules all 15 scenarios.
- C-04 (AC-3): manifest schema stays pinned to `kamn.e2e.evidence-manifest.v3`.
- C-05 (AC-3): manifest exposes run metadata markers (`run_id`, `started_at`, `completed_at`, `duration_seconds`, `execution_mode`).
- C-06 (AC-3): manifest infrastructure markers exist (`kolme_version`, `kamn_version`, `kamn_commit`, `kamn_agent_lib_version`, `agent_runtime`, `node_count`, `agent_count`, `storage_backend`).
- C-07 (AC-3): per-scenario manifest summary entries expose `id`, `name`, `status`, `duration_seconds`, `evidence_files`, `verifiable_outputs`.
- C-08 (AC-3): top-level manifest `summary` exposes totals for scenarios/proofs/messages/blocks.
- C-09 (AC-4): verifier report output includes deterministic markers for `schema_check`, `proof_check`, `chain_check`, `content_check`.
- C-10 (AC-4): verifier report is deterministic for identical input payloads.
- C-11 (AC-5): RED failures are observed before implementation and GREEN pass is recorded after implementation.
- C-12 (AC-6): phase-4a docs/research markers are present and internally coherent.
- C-13 (AC-7): fmt/clippy/tests pass.

## Success Metrics / Observable Signals
- `kamn-e2e-harness` contracts align with PRD section 7.1 scenario inventory and section 8.2/9 evidence-verification requirements.
- Deterministic report output can be consumed by downstream offline validation tooling.
- Phase-4 implementation slice starts from completed contract baselines instead of scaffolds.
