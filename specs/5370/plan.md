# Issue #5370 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5370` directionality markers.
- run that exact test before docs updates to capture RED evidence.

2. Add directionality contracts in daemon tests:
- add explicit directionality schema constants and helper assertions.
- add functional + integration tests for non-commutative host-pair extraction stability.

3. Extend docs marker contracts:
- add a `#5370` subsection in `docs/ops/configuration.md` with directionality markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include directionality hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: directionality assertions duplicate existing host-pair coverage without adding specificity.
  - Mitigation: require explicit extraction-rule marker and reverse-direction rejection assertions.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into runtime behavior.
  - Mitigation: keep changes test/docs-only.

## Interfaces / Contracts
- No production API changes.
- New contract: topology host-pair extraction is directionally canonical (`host_a->host_b`) and non-commutative.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
