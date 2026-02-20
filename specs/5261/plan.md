# Issue #5261 Plan

- Issue: #5261
- Milestone: specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md

## Approach
1. Extend `DataLayerPgExecutionAdapter` with blind-index search execution API and deterministic row decoding type.
2. Extend adapter with default RLS statement application API backed by bridge-projected statements.
3. Add report structures for RLS application and search results.
4. Add/expand tests for search success path, RLS apply path, and failure-path coverage.
5. Re-run targeted suites and quality gates.

## Affected Areas
- `crates/kamn-core/src/data_layer_postgres_execution_adapter.rs`
- `crates/kamn-core/src/lib.rs`
- `crates/kamn-core/tests/data_layer_postgres_execution_adapter.rs`
- `specs/5261/{spec.md,plan.md,tasks.md}`
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`
- `specs/milestones/r27-45-kamn-data-layer-prd-implementation-and-validation/index.md`
- `fixtures/ci/kamn_core_public_api_surface_baseline.env` (if public API surface changes)

## Risks and Mitigations
- Risk: Search decoding type mismatches (JSON/bytea/text) can fail at runtime.
  - Mitigation: explicit row decoding and regression tests on decode failures.
- Risk: RLS SQL application order drift causes nondeterministic behavior.
  - Mitigation: preserve bridge statement ordering and assert deterministic report output.
- Risk: Live DB-dependent tests become flaky.
  - Mitigation: keep env-gated live path and deterministic non-networked unit coverage.

## Interfaces / Contracts
- Add adapter method for blind-index search execution with requester DID context.
- Add adapter method for applying projected default RLS statements.
- Maintain fail-closed error taxonomy via `DataLayerPgExecutionAdapterError`.

## ADR
Not required; this extends established Phase-1 adapter design.
