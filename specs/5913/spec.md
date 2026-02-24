# Spec: Issue #5913 - Remove Compiled Fallback Signer Key Paths From kamn-core

- Issue: #5913
- Status: Reviewed
- Type: task
- Priority: P1
- Milestone: `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- Last Updated: 2026-02-24

## Problem Statement
`kamn-core` still resolves signer key material through a debug fallback path (`debug_fallback_signer_private_key_hex`) in `signer_backend.rs` and `transaction.rs`. This leaves signing behavior dependent on implicit process-generated key material rather than explicit operator provisioning.

## Scope
In scope:
- Remove runtime fallback key resolution in signer backend and transaction guard key-resolution paths.
- Require explicit key provisioning through configured signer private/public key environment variables.
- Add regression coverage for fail-closed behavior when key material is absent.

Out of scope:
- Signer provider architecture redesign.
- Protocol/wire-format changes.
- New dependency adoption.

## Acceptance Criteria
### AC-1 Signer backend key resolution is explicit-only
Given no signer key environment variables are set,
When local software backend signing is attempted,
Then signing fails with `SignerBackendError::MissingSignerKeyMaterial` and no fallback key is generated.

### AC-2 Transaction key resolution is explicit-only
Given no signer key environment variables are set,
When transaction signature verification relies on service-auth key material,
Then the transaction path does not source fallback key material and remains fail-closed.

### AC-3 Existing explicit-key paths remain stable
Given existing signer backend and transaction guard tests that provide explicit key material,
When fallback key paths are removed,
Then those tests remain green without behavioral regressions.

## Conformance Cases
- C-01 (AC-1, Unit/Regression): signer backend local path rejects missing key material with deterministic error.
- C-02 (AC-2, Unit/Regression): transaction key resolution helper returns `None` without explicit env and does not fallback.
- C-03 (AC-3, Integration): signer backend and transaction guard suites using explicit env key fixtures remain green.

## Success Metrics / Observable Signals
- `debug_fallback_signer_private_key_hex` is no longer used by signer backend/transaction key-resolution runtime paths.
- Local signing paths without explicit key config fail closed.
- Existing explicit-key integration paths continue to pass.
