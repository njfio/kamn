# R50.1 Milestone - Live-Postgres Runtime Bundle Integration

## Context
The live-postgres multi-host validation framework is currently represented in runtime output with a fixed row-count marker, while canonical selector rows remain test-fixture-only. This milestone promotes selector rows into production runtime contract data to reduce drift risk and strengthen live integration readiness.

## Scope
- Create a production source of truth for live-postgres multi-host execution bundle selector rows.
- Derive runtime row-count metadata from production selector data.
- Add deterministic contract checks to detect selector/row-count drift.

## Deliverables
- Issue #5471: runtime selector source and row-count derivation integration.

## Exit Criteria
- Runtime row-count marker is derived from production selector source length.
- Contract tests assert selector-row and runtime-marker consistency.
- Issue #5471 merged with spec status set to Implemented.
