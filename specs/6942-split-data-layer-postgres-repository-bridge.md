# 6942-split-data-layer-postgres-repository-bridge

## Objective
Split `crates/kamn-core/src/data_layer_postgres_repository_bridge.rs` into bounded, concern-based modules while preserving deterministic SQL operation projection, requester/session validation, and M5/M6/M7 capability-specific bridge behavior.

## Inputs/Outputs
- Inputs:
  - data-layer envelope, query, telemetry, graph, and embedding records
  - requester/session DID inputs
  - M5 pgvector capability config
  - M6 AGE capability config
  - M7 Timescale capability config
- Outputs:
  - unchanged postgres repository bridge semantics
  - a thin root shell in `data_layer_postgres_repository_bridge.rs`
  - bounded sibling modules for operation models, requester/session projection, M5 helpers, M6 helpers, M7 helpers, and tests/support where needed
  - a hard-fail extraction contract for the root shell and module layout

## Boundaries/Non-goals
- No changes to SQL statement text or bind-order markers
- No changes to stable reason codes or capability gating behavior
- No new dependencies
- No unrelated data-layer refactors outside the postgres repository bridge surface

## Failure modes
- invalid requester DID remains fail-closed
- invalid owner DID remains fail-closed
- pgvector extension unavailability remains fail-closed
- pgvector dimension mismatch remains fail-closed
- AGE extension unavailability remains fail-closed
- unsupported AGE relation projection remains fail-closed
- Timescale extension unavailability remains fail-closed
- invalid Timescale bucket window inputs remain fail-closed
- extraction contract fails if the root shell or module layout regress

## Acceptance criteria
- [ ] `crates/kamn-core/src/data_layer_postgres_repository_bridge.rs` becomes a thin root shell under the active file-size budget
- [ ] bounded sibling modules separate operation models, requester/session projection, M5 pgvector helpers, M6 AGE helpers, M7 Timescale helpers, and tests/support where appropriate
- [ ] a hard-fail extraction contract enforces the root shell and module layout
- [ ] existing postgres repository bridge tests remain green without semantic drift
- [ ] touched-Rust size policy returns `policy_decision=GO`
- [ ] final spec records test evidence and any deviations

## Files to touch
- `crates/kamn-core/src/data_layer_postgres_repository_bridge.rs`
- `crates/kamn-core/src/data_layer_postgres_repository_bridge/`
- `crates/kamn-core/tests/data_layer_postgres_repository_bridge_module_extraction_contract.rs`
- `specs/6942-split-data-layer-postgres-repository-bridge.md`

## Error semantics
- Preserve existing typed error behavior and stable reason markers
- Preserve fail-closed validation for requester DIDs, owner DIDs, capability availability, dimension checks, relation checks, and bucket-window validation
- Do not introduce silent fallbacks or relaxed capability gating

## Test plan
- Add a red extraction contract that fails while `data_layer_postgres_repository_bridge.rs` remains monolithic
- Run the extraction contract green once the split is in place
- Run the real repository bridge tests after extraction
- Run touched-Rust size policy against the staged write set
