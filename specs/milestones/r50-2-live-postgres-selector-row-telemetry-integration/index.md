# R50.2 Milestone - Live-Postgres Selector Row Telemetry Integration

## Context
R50.1 promoted live-postgres multi-host selector rows into runtime orchestration for row-count derivation and drift checks, but daemon runtime telemetry still omits the canonical selector-row CSV marker at execution-complete boundaries.

## Scope
- Emit selector-row CSV marker in daemon runtime completion telemetry.
- Guarantee marker value is derived from runtime selector-row source.
- Add deterministic test coverage for selector-row telemetry coherence.

## Deliverables
- Issue #5473 runtime telemetry marker integration and contract checks.

## Exit Criteria
- `multi_host_execution_bundle_selector_rows_csv` appears in daemon runtime completion logs.
- Marker value and row-count/prefix markers remain coherent.
- Issue #5473 merged and spec marked Implemented.
