# Issue #5348 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5348` load-profile markers.
- run that exact test before docs updates to capture RED evidence.

2. Add bounded load-profile matrix contracts in daemon tests:
- define canonical load-profile matrix constants and helpers in `daemon_tests`.
- add functional + integration tests asserting deterministic reason/taxonomy outcomes for applied/deferred scenarios across profiles.

3. Extend docs marker contracts:
- add a `#5348` subsection in `docs/ops/configuration.md` with load-profile CSV/order markers and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include load-profile hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: load-profile matrix values become flaky under env-gated paths.
  - Mitigation: keep profiles bounded and deterministic; preserve env-unset short-circuit behavior.
- Risk: contract duplication drift between docs and tests.
  - Mitigation: dedicated docs-contract assertions for exact marker values and command references.
- Risk: over-broad scope drift toward full load testing.
  - Mitigation: constrain to bounded profile matrix and deterministic reason/taxonomy assertions only.

## Interfaces / Contracts
- No production API changes.
- New contract: bounded load-profile matrix for live-postgres applied/deferred scenarios must remain deterministic and taxonomy-consistent.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
