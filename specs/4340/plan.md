# Plan — #4340

Status: Reviewed

## Approach

- Deliver #4343 first (graduation governance hardening), then proceed to #4344 in a follow-up chain.
- Keep changes scoped to existing missing-docs checker/test/doc contracts in `scripts/ci`, `fixtures/ci`, and docs contract references.
- Ensure all added governance outputs are deterministic and easy to parse in CI logs.

## Affected Areas

- `scripts/ci/check_kamn_core_missing_docs_policy.sh`
- `scripts/ci/test_check_kamn_core_missing_docs_policy.sh`
- `docs/ci/strategy.md`
- `specs/4343/*`, `specs/4349/*`, `specs/4350/*`

## Risks and Mitigations

- Risk: checker output changes break downstream log expectations.
  - Mitigation: additive markers only; preserve existing pass/fail contract line.
- Risk: over-broad policy reasons become unstable.
  - Mitigation: emit fixed taxonomy/version markers and explicit count/delta fields.

## Interfaces and Contracts

- Shell checker output (stdout/stderr marker lines) is treated as the governance contract.
- Existing velocity policy schema from `missing_docs_velocity_guard.py` is source-of-truth for delta values.
