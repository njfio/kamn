# Plan — Issue #4182

## Approach

1. Extend `scripts/kolme/test_run_version_compatibility_contract_lane.sh` with explicit red fixtures
   for taxonomy drift and runbook marker divergence.
2. Reuse existing contract lane entrypoint to capture fail-closed reason assertions.
3. Add deterministic assertions for the exact mismatch reason codes.

## Affected Modules

- `scripts/kolme/test_run_version_compatibility_contract_lane.sh`

## Risks / Mitigations

- Risk: fixture coupling to docs marker strings could become brittle.
  Mitigation: assert stable contract markers only (command + taxonomy markers), not prose text.

## Interfaces / Contracts

- Contract lane must emit deterministic mismatch reason codes for taxonomy/runbook drift.

## ADR

- Not required (no architecture or protocol decision changes).
