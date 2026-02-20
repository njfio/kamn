# Issue #5358 Plan

## Implementation Approach
1. Add docs-contract RED gate:
- add a new docs-contract test for `#5358` order-invariance markers.
- run that exact test before docs updates to capture RED evidence.

2. Add order-invariance contracts in daemon tests:
- add deterministic helpers that normalize lane outputs into stable fingerprints.
- add functional + integration tests asserting fingerprint equivalence between baseline and permuted lane execution orders.

3. Extend docs marker contracts:
- add a `#5358` subsection in `docs/ops/configuration.md` with order-invariance marker semantics and validation command references.

4. Update review narrative:
- refine `docs/review/gaps-and-issues-r45.md` next-frontier wording to include order-invariance hardening.

## Affected Modules
- `crates/kamn-node/src/main_tests/daemon_tests.rs` (test-only)
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs` (test-only)
- `docs/ops/configuration.md`
- `docs/review/gaps-and-issues-r45.md`

## Risks and Mitigations
- Risk: order-invariance assertions could accidentally depend on unstable map iteration order.
  - Mitigation: use explicitly sorted fingerprints for deterministic comparisons.
- Risk: docs/test marker drift.
  - Mitigation: dedicated docs-contract assertions for exact marker strings and commands.
- Risk: scope creep into multi-host scheduling.
  - Mitigation: constrain to same-host bounded lane permutations only.

## Interfaces / Contracts
- No production API changes.
- New contract: bounded parallel lane reason/taxonomy fingerprints must be invariant to lane execution order.

## ADR
- Not required: no dependency/protocol/architecture behavior changes.
