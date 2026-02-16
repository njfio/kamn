# Issue #3915 Plan

- Issue: `#3915`
- Status: `Completed`

## Approach
- Add a dedicated policy-contract test target for signer secret-lifecycle checks.
- Encode required lifecycle marker set and forbidden fallback reason code in checker helper.
- Add docs parity assertions against:
  - `docs/ci/strategy.md`
  - `docs/plans/2026-02-14-production-service-next-steps.md`
- Keep runtime behavior unchanged and verify with scoped signer suites.

## Affected Modules
- `crates/kamn-node/tests/signer_secret_lifecycle_policy_contract.rs`
- `docs/ci/strategy.md`
- `docs/plans/2026-02-14-production-service-next-steps.md`

## Risks and Mitigations
- Risk: marker set drifts silently.
- Mitigation: docs/source parity checks fail closed on missing markers.
- Risk: policy tests diverge from runtime terminology.
- Mitigation: checker constants use existing runtime marker/reason code strings.

## Interface Contract
- No public API changes.
- Test/docs policy contract additions only.

## ADR
- No ADR required.
