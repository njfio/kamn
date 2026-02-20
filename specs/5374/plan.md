# Issue #5374 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5374` lane-set mapping markers.
- run that exact test before docs updates to capture RED evidence.

2. Add mapping contracts in daemon tests:
- add explicit topology-id->lane-set mapping constants/helper assertions.
- add functional + integration tests for mapping stability and permutation invariance.

3. Extend docs marker contracts:
- add a `#5374` subsection in `docs/ops/configuration.md` with mapping markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include lane-set mapping hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: mapping assertions duplicate topology scope checks.
  - Mitigation: enforce explicit topology-id keyed lane-set mapping rows with fail-closed docs markers.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into runtime behavior.
  - Mitigation: keep changes test/docs-only.

## Interfaces / Contracts
- No production API changes.
- New contract: topology ids map to canonical lane-set classes and remain stable under permutations.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
