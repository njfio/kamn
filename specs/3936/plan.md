# Issue #3936 Plan

- Issue: #3936
- Status: Implemented
- Spec: `specs/3936/spec.md`

## Delivery Approach
1. Execute panic-path retirement in two focused subtasks:
   - `#3941`: remove `unreachable!()` path and add signer-source guard.
   - `#3940`: harden production-source extraction for `expect(` guard and broaden runtime coverage.
2. Keep runtime watchdog docs aligned with retirement mapping markers.
3. Verify with scoped node/core tests plus strict lint/format checks.

## Affected Modules
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/cli_tests.rs`
- `docs/foundation/runtime-watchdog-attestation.md`

## Risks and Mitigations
- Risk: false-negative source extraction around top-level `#[cfg(test)]` attributes.
  - Mitigation: cfg(test)-item skipping extraction parser + regression fixture (`#3940`).
- Risk: panic primitive reintroduction in signer module.
  - Mitigation: signer-source macro regression guard (`#3941`).

## Contracts and Interfaces
- Panic-path production guard contract for scoped runtime files:
  - no `expect(`, `unreachable!(`, `panic!(` in production regions.
- Signer decode-failure contract remains typed (`ConfigError::RuntimeKolmeLive`).
- Retirement mapping remains documented in runtime watchdog attestation docs.

## Verification Strategy
- RED/GREEN/REGRESSION evidence is captured in child PRs:
  - `#5153` for `#3941`
  - `#5154` for `#3940`
- Parent closeout verifies AC and conformance mapping completeness.
