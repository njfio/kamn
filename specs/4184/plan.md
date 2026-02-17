# Plan — Issue #4184

## Approach

1. Extend existing deploy go/no-go bundle tests with explicit red fixtures for upgrade rehearsal
   lineage completeness and tamper rejection.
2. Add deterministic assertions for reason taxonomy/version/csv and specific fail-closed reasons.
3. Keep fixture coverage scoped to existing script lanes to avoid new lane sprawl.

## Affected Modules

- `scripts/deploy/test_generate_gonogo_evidence_bundle.sh`

## Risks / Mitigations

- Risk: red fixtures become brittle to docs prose edits.
  Mitigation: assert only machine-readable markers and deterministic reason codes.

## Interfaces / Contracts

- upgrade lineage checks must return deterministic reason taxonomy and reason code set/order.

## ADR

- Not required.
