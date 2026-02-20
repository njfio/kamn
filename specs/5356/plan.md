# Issue #5356 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5356` asymmetric lane markers.
- run that exact test before docs updates to capture RED evidence.

2. Add asymmetric parallel lane contracts in daemon tests:
- define canonical asymmetric parallel lane constants/helpers in `daemon_tests`.
- add functional + integration tests asserting deterministic reason/taxonomy outcomes for applied/deferred lanes with mixed leg cadence.

3. Extend docs marker contracts:
- add a `#5356` subsection in `docs/ops/configuration.md` with asymmetric lane CSV/order markers and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include asymmetric parallel lane hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: mixed-cadence lanes increase concurrency flake potential.
  - Mitigation: keep lane count bounded, deterministic, and env-gated.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into multi-host orchestration.
  - Mitigation: explicitly constrain to same-host asymmetric cadence parallel lanes.

## Interfaces / Contracts
- No production API changes.
- New contract: bounded asymmetric parallel lanes for live-postgres applied/deferred scenarios must remain deterministic and taxonomy-consistent.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
