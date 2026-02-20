# Issue #5352 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5352` role-pair markers.
- run that exact test before docs updates to capture RED evidence.

2. Add role-pair matrix contracts in daemon tests:
- define canonical role-pair matrix constants and helpers in `daemon_tests`.
- add functional + integration tests asserting deterministic reason/taxonomy outcomes for ordered applied/deferred role-pair runs.

3. Extend docs marker contracts:
- add a `#5352` subsection in `docs/ops/configuration.md` with role-pair CSV/order markers and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include role-pair hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: ordered role-pair execution semantics could be interpreted as true distributed coverage.
  - Mitigation: explicitly label as bounded distributed-lane precursor and keep scope test-contract only.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact role-pair marker strings and commands.
- Risk: scope creep into networking orchestration.
  - Mitigation: limit scope to ordered two-node role-pair daemon runs without network topology changes.

## Interfaces / Contracts
- No production API changes.
- New contract: ordered two-node role-pair matrix for live-postgres applied/deferred scenarios must remain deterministic and taxonomy-consistent.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
