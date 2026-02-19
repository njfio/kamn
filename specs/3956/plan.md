# Issue #3956 Plan

- Issue: #3956
- Status: Completed
- Spec: `specs/3956/spec.md`

## Implementation Approach
1. Add a typed rotation-freshness outcome model in `signer_policy.rs`.
2. Extend freshness evaluation to reject non-failover rotation metadata regressions in addition to existing failover stale checks.
3. Preserve deterministic reason-code behavior and add new reason taxonomy marker for non-failover regression.
4. Add unit/functional/integration/regression coverage for freshness matrix and stale metadata rejection.
5. Update runtime-network/watchdog/ops docs and docs-contract assertions for freshness reason markers.

## Affected Modules
- `crates/kamn-node/src/signer/signer_policy.rs`
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `crates/kamn-node/tests/signer_policy_reason_taxonomy_contract.rs`
- `docs/foundation/runtime-network.md`
- `docs/foundation/runtime-watchdog-attestation.md`
- `crates/kamn-core/tests/runtime_watchdog_attestation_docs.rs`
- `docs/ops/configuration.md`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`

## Risks and Mitigations
- Risk: false-positive stale rejection in non-failover flows.
  - Mitigation: encode explicit outcome matrix with unit tests covering fresh/stale boundaries.
- Risk: reason-taxonomy drift across source/docs/contracts.
  - Mitigation: update signer-policy taxonomy contract + watchdog docs contract in same change.
- Risk: signer module regression.
  - Mitigation: use existing preflight evaluation path; no transport/backend flow expansion.

## Contracts and Interfaces
- New typed outcome contract:
  - `SignerRotationFreshnessOutcome` (fresh / stale-failover / stale-non-failover-regressed).
- Deterministic reason markers:
  - Existing: `runtime_signer_rotation_epoch_stale`
  - New: `runtime_signer_rotation_epoch_regressed`

## Verification Strategy
- Unit: rotation freshness outcome matrix.
- Functional: signer preflight non-failover stale metadata rejection.
- Integration: main test harness stale metadata matrix check.
- Regression: signer-policy reason-taxonomy contract and stale failover guard parity.
- Docs: runtime-watchdog attestation marker assertions.
