# Issue #5354 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5354` parallel role-pair markers.
- run that exact test before docs updates to capture RED evidence.

2. Add bounded parallel role-pair lane contracts in daemon tests:
- define canonical parallel lane constants/helpers in `daemon_tests`.
- add functional + integration tests asserting deterministic reason/taxonomy outcomes for applied/deferred lanes where role pair legs execute concurrently.

3. Extend docs marker contracts:
- add a `#5354` subsection in `docs/ops/configuration.md` with parallel lane CSV/order markers and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include bounded parallel lane hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: parallel test execution introduces flakiness from shared process state.
  - Mitigation: keep lane count bounded and deterministic; preserve env-gated short-circuit path.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into full distributed networking.
  - Mitigation: limit scope to bounded same-host parallel role-pair execution only.

## Interfaces / Contracts
- No production API changes.
- New contract: bounded parallel role-pair lanes for live-postgres applied/deferred scenarios must remain deterministic and taxonomy-consistent.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
