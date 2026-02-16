# Issue #3807 Plan

- Issue: `#3807`
- Status: `InProgress`

## Approach
- Introduce a dedicated signer-policy taxonomy contract test target under `crates/kamn-node/tests`.
- Guard two surfaces:
  - signer policy source markers (`src/signer/signer_policy.rs`)
  - runtime-network docs markers (`docs/foundation/runtime-network.md`)
- Add signer-policy taxonomy section in runtime-network docs with deterministic marker list.
- Run scoped signer and docs tests as regression gates.

## Affected Modules
- `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- `docs/foundation/runtime-network.md`
- `specs/3807/spec.md`
- `specs/3807/tasks.md`

## Risks and Mitigations
- Risk: taxonomy list drifts from source updates.
- Mitigation: source/doc parity contract test fails closed on missing markers.
- Risk: over-broad marker assertions create brittle test churn.
- Mitigation: constrain list to signer-policy fail-closed reasons required by subtask scope.

## Interface Contract
- No runtime API changes.
- New test-only contract:
  - signer-policy source must retain required reason markers.
  - runtime-network docs must retain same marker set and taxonomy version marker.

## ADR
- No ADR required: this subtask adds verification/docs parity only and introduces no new dependencies or protocol changes.
