# Issue #5360 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5360` multi-permutation markers.
- run that exact test before docs updates to capture RED evidence.

2. Add multi-permutation invariance contracts in daemon tests:
- add deterministic permutation helpers (reverse/rotate/interleaved) for lane vectors.
- add functional + integration tests asserting sorted-fingerprint equivalence across these permutations for symmetric/asymmetric lane sets.

3. Extend docs marker contracts:
- add a `#5360` subsection in `docs/ops/configuration.md` with permutation marker semantics and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include permutation-invariance hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: permutation helpers could become non-deterministic if implemented via hash-based ordering.
  - Mitigation: implement explicit vector transforms only.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into stochastic fuzzing.
  - Mitigation: constrain to small deterministic permutation set.

## Interfaces / Contracts
- No production API changes.
- New contract: bounded parallel lane fingerprints must remain invariant across canonical deterministic permutations.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
