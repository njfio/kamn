# Issue #3947 Plan

- Issue: #3947
- Status: Completed
- Spec: `specs/3947/spec.md`

## Implementation Approach
1. Add a dedicated `kamn-core` docs-contract test for node runtime test ownership markers and guard commands.
2. Add deterministic ownership taxonomy/status markers to `runtime-watchdog-attestation.md`.
3. Add CI strategy guard command marker for the new docs-contract test.
4. Run targeted docs suites.

## Affected Modules
- `crates/kamn-core/tests/node_test_surface_ownership_docs.rs`
- `docs/foundation/runtime-watchdog-attestation.md`
- `docs/ci/strategy.md`

## Risks and Mitigations
- Risk: docs wording refactors break tests unnecessarily.
  - Mitigation: assert stable marker keys/paths/commands rather than prose.
- Risk: ownership section diverges from module layout.
  - Mitigation: require explicit shell + fragment path markers.

## Contracts and Interfaces
- Reason taxonomy version: `kamn.node.runtime-test-ownership-reason-taxonomy.v1`.
- Required ownership markers and guard command are deterministic contract surface.

## Verification Strategy
- RED: run new docs-contract test before adding ownership markers (expected fail).
- GREEN: add markers/docs command references and rerun targeted docs tests.
- REGRESSION: run `runtime_watchdog_attestation_docs` and `ci_strategy_docs`.
