# Issue #5362 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5362` fingerprint schema markers.
- run that exact test before docs updates to capture RED evidence.

2. Add fingerprint-schema contracts in daemon tests:
- add explicit fingerprint schema constants and a helper for deterministic formatting.
- add functional + integration tests for schema structure and repeated-run stability.

3. Extend docs marker contracts:
- add a `#5362` subsection in `docs/ops/configuration.md` with fingerprint schema/version/field-order markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include fingerprint-schema hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: schema parsing assertions become brittle with accidental delimiter changes.
  - Mitigation: codify delimiter/field order explicitly and assert exact shape.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into runtime output refactors.
  - Mitigation: keep scope on test/docs contracts only.

## Interfaces / Contracts
- No production API changes.
- New contract: parallel lane fingerprints must conform to canonical schema version and field-order across repeated runs.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
