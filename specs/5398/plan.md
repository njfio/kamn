# Issue #5398 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5398` host/lane fingerprint-hash order-normalization markers.
- run that exact test before docs updates to capture RED evidence.

2. Add order-normalization contracts in daemon tests:
- add explicit topology-id->host-mode->host-pair->lane-set->lane-fingerprint-hash order-normalization constants/helper assertions.
- add functional + integration tests for sorted-row stability and permutation invariance.

3. Extend docs marker contracts:
- add a `#5398` subsection in `docs/ops/configuration.md` with order-normalization markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include host/lane fingerprint-hash order-normalization hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: order-normalization assertions overlap with existing fingerprint-hash coherence surface.
  - Mitigation: enforce explicit canonical sorted row contract string distinct from coherence-only assertions.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into runtime behavior.
  - Mitigation: keep changes test/docs-only.

## Interfaces / Contracts
- No production API changes.
- New contract: topology ids map to canonical sorted host/lane fingerprint-hash rows and remain stable under permutations.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
