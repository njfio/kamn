# Issue #5368 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5368` host-pair markers.
- run that exact test before docs updates to capture RED evidence.

2. Add host-pair contracts in daemon tests:
- add explicit host-pair schema constants and extraction helper.
- add functional + integration tests for deterministic host-pair mapping and permutation invariance.

3. Extend docs marker contracts:
- add a `#5368` subsection in `docs/ops/configuration.md` with host-pair markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include host-pair hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: host-pair assertions become order-sensitive to incidental fingerprint ordering.
  - Mitigation: sort fingerprints and assert canonical host-pair id set.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into runtime behavior.
  - Mitigation: keep changes test/docs-only.

## Interfaces / Contracts
- No production API changes.
- New contract: topology fingerprints must expose canonical host-pair ids that remain stable under repeated runs and topology permutations.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
