# Issue #3955 Plan

- Issue: #3955
- Status: Completed
- Spec: `specs/3955/spec.md`

## Implementation Approach
1. Introduce a managed key-source adapter abstraction in `signer/managed_backend.rs` that returns signature output plus a deterministic provenance marker payload.
2. Add signer-side parity validation that consumes the provenance marker against resolved signer selection (profile, key source, and key-reference env).
3. Route `build_kolme_live_direct_signed_wire_payload` managed-external flow through the adapter abstraction.
4. RED: add targeted tests for deterministic marker emission and mismatch fail-closed behavior.
5. GREEN: wire adapter + parity checks with minimal changes.
6. REGRESSION: rerun targeted managed-signer tests and docs-contract suite.

## Affected Modules
- `crates/kamn-node/src/signer/managed_backend.rs`
- `crates/kamn-node/src/signer.rs`
- `crates/kamn-node/src/main_tests/signer_tests.rs`
- `crates/kamn-core/tests/service_api_ops_configuration_docs.rs`
- `docs/ops/configuration.md`

## Risks and Mitigations
- Risk: regressions in existing managed-external signing path.
  - Mitigation: keep legacy signing/provenance validation intact and add adapter/parity tests over current behavior.
- Risk: doc-contract drift from new mapping marker text.
  - Mitigation: add explicit docs-contract assertion for new marker and keep marker format deterministic.
- Risk: signer module growth.
  - Mitigation: keep adapter logic in `managed_backend.rs`; use signer-side parity helper only.

## Contracts and Interfaces
- Adapter output contract (managed-external):
  - `canonical_message`
  - `signature_hex`
  - `recovery_id`
  - `provenance_marker` with deterministic:
    - `profile`
    - `key_source`
    - `key_reference_env`
    - `signer_public_key_hex`
- Parity enforcement contract:
  - mismatch between marker selection fields and resolved signer selection fails closed with deterministic reason code.

## Verification Strategy
- Unit: deterministic provenance marker emission fields.
- Functional: managed adapter success path marker parity.
- Integration: managed profile execution path through runtime signer builder.
- Regression: mismatch marker rejects with deterministic reason.
- Docs: `docs/ops/configuration.md` marker mapping contract assertion.
