# Issue #5366 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5366` topology permutation markers.
- run that exact test before docs updates to capture RED evidence.

2. Add topology permutation contracts in daemon tests:
- add explicit topology permutation constants + permutation helper.
- add functional + integration tests for topology permutation invariance.

3. Extend docs marker contracts:
- add a `#5366` subsection in `docs/ops/configuration.md` with topology permutation markers and validation commands.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include topology permutation hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: permutation helper introduces accidental non-deterministic ordering.
  - Mitigation: deterministic baseline/reverse/rotate permutations with explicit functional assertions.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into runtime behavior.
  - Mitigation: keep changes test/docs-only.

## Interfaces / Contracts
- No production API changes.
- New contract: canonical topology profile permutations must preserve sorted topology fingerprint bundles.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
