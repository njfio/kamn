# Issue #5378 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5378` host-mode mapping markers.
- run that exact test before docs updates to capture RED evidence.

2. Add mapping contracts in daemon tests:
- add explicit topology-id->host-mode mapping constants/helper assertions.
- add functional + integration tests for mapping stability and permutation invariance.

3. Extend docs marker contracts:
- add a `#5378` subsection in `docs/ops/configuration.md` with mapping markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include host-mode mapping hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: mapping assertions overlap with host-pair/host-directionality coverage.
  - Mitigation: enforce explicit topology-id keyed host-mode rows independent of host-pair row strings.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into runtime behavior.
  - Mitigation: keep changes test/docs-only.

## Interfaces / Contracts
- No production API changes.
- New contract: topology ids map to canonical host modes (`same_host`, `distributed_label`) and remain stable under permutations.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
