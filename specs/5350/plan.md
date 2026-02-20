# Issue #5350 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5350` role-profile markers.
- run that exact test before docs updates to capture RED evidence.

2. Add role-profile matrix contracts in daemon tests:
- define canonical role-profile matrix constants and helpers in `daemon_tests`.
- add functional + integration tests asserting deterministic reason/taxonomy outcomes for applied/deferred role profiles.

3. Extend docs marker contracts:
- add a `#5350` subsection in `docs/ops/configuration.md` with role-profile CSV/order markers and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include role-profile hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: role-specific execution drift complicates deterministic assertions.
  - Mitigation: keep role matrix bounded to canonical applied/deferred profiles and assert existing deterministic reason/taxonomy contracts only.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact role-profile marker strings and commands.
- Risk: scope creep into multi-node orchestration.
  - Mitigation: limit scope to single-process role-profile matrix only.

## Interfaces / Contracts
- No production API changes.
- New contract: bounded role-profile matrix for live-postgres applied/deferred scenarios must remain deterministic and taxonomy-consistent.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
