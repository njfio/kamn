# Issue #5390 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5390` host/lane composite coherence markers.
- run that exact test before docs updates to capture RED evidence.

2. Add coherence contracts in daemon tests:
- add explicit topology-id->host-mode->host-pair->lane-set->lane-count coherence constants/helper assertions.
- add functional + integration tests for coherence stability and permutation invariance.

3. Extend docs marker contracts:
- add a `#5390` subsection in `docs/ops/configuration.md` with coherence markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include host/lane composite coherence hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: composite coherence assertions overlap with partial mapping/coherence surfaces.
  - Mitigation: enforce explicit quintuple rows (`topology_id->host_mode->host_pair->lane_set->lane_count`) as a distinct contract surface.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into runtime behavior.
  - Mitigation: keep changes test/docs-only.

## Interfaces / Contracts
- No production API changes.
- New contract: topology ids map to canonical host/lane composite coherence rows and remain stable under permutations.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
