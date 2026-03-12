# 6944-split-data-layer-m7-timeseries-telemetry

## Objective
Split `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs` into bounded, concern-based modules while preserving deterministic M7 telemetry ingest, hourly/daily aggregation, owner billing projection/reconciliation, observability projection, and stable authorization/error semantics.

## Inputs/Outputs
- Inputs:
  - owner/agent-scoped M7 telemetry sample inputs
  - requester owner DID and owner-scope aggregate queries
  - billing reconciliation inputs
  - observability projection inputs derived from telemetry samples
- Outputs:
  - unchanged M7 telemetry registry, billing, and observability semantics
  - a thin root shell in `data_layer_m7_timeseries_telemetry.rs`
  - bounded sibling modules for models/constants, registry logic, billing projection/reconciliation, observability projection, support/error helpers, and tests
  - a hard-fail extraction contract for the root shell and module layout

## Boundaries/Non-goals
- No changes to stable reason codes
- No changes to owner-scope authorization semantics
- No changes to billing reconciliation decisions or aggregate math
- No new dependencies
- No unrelated data-layer refactors outside the M7 telemetry surface

## Failure modes
- invalid DIDs remain fail-closed
- owner-scope authorization remains fail-closed
- invalid observability samples remain fail-closed with the existing reason markers
- extraction contract fails if the root shell or module layout regress

## Acceptance criteria
- [ ] `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs` becomes a thin root shell under the active file-size budget
- [ ] bounded sibling modules separate models/constants, registry logic, billing projection/reconciliation, observability projection, support/error helpers, and tests
- [ ] a hard-fail extraction contract enforces the root shell and module layout
- [ ] existing M7 time-series telemetry tests remain green without semantic drift
- [ ] touched-Rust size policy returns `policy_decision=GO`
- [ ] final spec records test evidence and any deviations

## Files to touch
- `crates/kamn-core/src/data_layer_m7_timeseries_telemetry.rs`
- `crates/kamn-core/src/data_layer_m7_timeseries_telemetry/`
- `crates/kamn-core/tests/data_layer_m7_timeseries_telemetry_module_extraction_contract.rs`
- `specs/6944-split-data-layer-m7-timeseries-telemetry.md`

## Error semantics
- Preserve existing typed error behavior and stable reason markers
- Preserve fail-closed validation for owner/requester DIDs and observability sample projection
- Do not introduce silent fallbacks or relaxed authorization behavior

## Test plan
- Add a red extraction contract that fails while `data_layer_m7_timeseries_telemetry.rs` remains monolithic
- Run the extraction contract green once the split is in place
- Run the real M7 time-series telemetry tests after extraction
- Run touched-Rust size policy against the staged write set
