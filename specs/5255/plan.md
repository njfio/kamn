# Issue #5255 Plan

- Issue: #5255
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Introduce migration scaffolding with one baseline SQL file containing core table/index/policy markers needed by Phase 1.
2. Add a dedicated migration contract test suite that reads the SQL file and asserts required markers.
3. Keep runtime behavior unchanged; this task only establishes schema artifacts and deterministic guardrails.
4. Update planning/review docs to bind this task to the R27.45 infrastructure-activation track.

## Affected Areas
- `crates/kamn-core/migrations/`
- `crates/kamn-core/tests/data_layer_postgres_migration_contract.rs`
- `docs/review/data-layer-roadmap.md`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`

## Risks and Mitigations
- Risk: marker assertions become too brittle and block harmless formatting updates.
  - Mitigation: assert stable semantic markers (table/index/policy identifiers), not whole SQL fragments.
- Risk: schema naming drifts from PRD module language.
  - Mitigation: encode required identifiers directly in tests and docs with explicit references.
- Risk: accidental shell-surface growth while adding validation.
  - Mitigation: use Rust tests only; no new shell scripts/workflow files.

## Interfaces / Contracts
- Baseline migration file must include these table identifiers:
  - `messages`
  - `merkle_batches`
  - `did_registry`
  - `escrows`
  - `key_rotation_log`
  - `access_log`
- Baseline migration must include deterministic marker comments:
  - `-- KAMN_M2_RLS_MARKER:...`
  - `-- KAMN_M3_INDEX_MARKER:...`
  - `-- KAMN_M8_RETENTION_MARKER:...`

## ADR
Not required (bootstrap schema scaffolding and contract checks only).
