# Issue 6234 Spec

Status: Reviewed
Priority: P0
Milestone: R59 Swarm Gap Closure
Parent: #6223

## Problem Statement
The observability endpoint TLS resolver defaults to `Disabled` when TLS mode env is absent. This allows plaintext observability exposure in production-safe runtime flows unless operators remember to set explicit TLS env flags.

## Scope
In scope:
- Make observability TLS default fail-closed (`Require`) for production-safe runtime mode `kolme-live`.
- Preserve explicit local/dev override semantics via `KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE=disabled`.
- Add tests for default + override behavior and fail-closed error markers.
- Update runtime network documentation for the new default behavior.

Out of scope:
- Reworking service API TLS policy.
- Changing observability endpoint route schemas/payload contracts.
- Introducing new TLS configuration environment variables.

## Acceptance Criteria
- AC-1: When runtime mode is `kolme-live` and TLS mode env is absent, resolver defaults to `Require` and fails closed if cert/key env vars are missing.
- AC-2: Explicit `KAMN_OBSERVABILITY_ENDPOINT_TLS_MODE=disabled` keeps local/dev plaintext behavior available.
- AC-3: Existing explicit `require` mode behavior remains intact (cert/key validation + deterministic failures).
- AC-4: Runtime network docs reflect `kolme-live` default TLS requirement and local/dev override semantics.

## Conformance Cases
- C-01 (AC-1, Conformance): Integration test verifies `kolme-live` snapshot fails with deterministic missing-cert env marker when no TLS env is provided.
- C-02 (AC-2, Conformance): Integration test verifies explicit `disabled` mode still serves HTTP observability responses for `kolme-live`.
- C-03 (AC-3, Regression): Existing TLS require-mode tests continue passing (required HTTPS route + invalid cert/key/mode failure paths).
- C-04 (AC-4, Functional): Runtime network doc declares default/override contract markers aligned with resolver behavior.

## Success Metrics
- Production-safe observability path (`kolme-live`) no longer silently defaults to plaintext.
- Local/dev plaintext mode remains explicit and intentional.
- Drift between runtime behavior and docs is covered by deterministic tests.
