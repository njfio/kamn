# Issue #5344 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5344` taxonomy/ordering markers.
- run that exact test before docs updates to capture RED evidence.

2. Add canonical matrix projection contract in daemon tests:
- define canonical matrix taxonomy constants and row projection helper in `daemon_tests`.
- add one focused functional test that asserts ordering and reason mapping deterministically.

3. Extend docs marker contracts:
- add a `#5344` subsection in `docs/ops/configuration.md` with taxonomy version, reason-codes CSV, scenario-order CSV, and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include taxonomy/ordering contract hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: marker drift between docs and test constants.
  - Mitigation: dedicated docs-contract assertions for exact marker strings.
- Risk: matrix helper complexity growth.
  - Mitigation: keep helper minimal and deterministic with static row values.
- Risk: env-gated behavior regressions.
  - Mitigation: reuse existing reason constants; matrix projection remains independent from live DB availability.

## Interfaces / Contracts
- No production API changes.
- New contract: live-postgres daemon matrix taxonomy/version and canonical scenario order markers must remain stable and synchronized between code tests and docs.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
