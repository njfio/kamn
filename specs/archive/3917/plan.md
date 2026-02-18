# Issue #3917 Plan

- Issue: `#3917`
- Status: `Completed`

## Approach
- Add docs-contract assertions for signer secret-lifecycle policy markers that must remain stable.
- Enforce closure-chain marker parity in production next-steps planning docs.
- Keep checks deterministic and fail closed on missing or renamed markers.

## Affected Modules
- `crates/kamn-node/tests/signer_secret_lifecycle_policy_contract.rs`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`

## Risks and Mitigations
- Risk: docs marker drift silently weakens policy governance.
- Mitigation: explicit marker list and guard-command assertions in contract tests.
- Risk: closure chain metadata diverges from issue hierarchy.
- Mitigation: enforce deterministic chain marker strings in docs-contract test.

## Interface Contract
- No runtime API changes.
- Docs-contract coverage only.

## ADR
- No ADR required.
