# Spec: Issue 6185 - Service API TLS Default Hardening

- Issue: #6185
- Milestone: `R59 Swarm Gap Closure`
- Status: Implemented
- Priority: P0
- Area: security

## Problem Statement

Service API TLS mode currently defaults to `Disabled` when `KAMN_SERVICE_API_TLS_MODE`
is not set. This leaves plaintext transport as the out-of-box behavior.

## Scope

In scope:
1. Make production-mode default fail closed unless TLS is explicitly configured.
2. Preserve deterministic insecure default only for test builds to keep harness stability.
3. Preserve explicit `KAMN_SERVICE_API_TLS_MODE=disabled` behavior.

Out of scope:
1. Observability endpoint TLS policy changes.
2. Automatic certificate provisioning.
3. Service API auth model changes.

## Acceptance Criteria

### AC-1 Production Default Fails Closed
Given TLS mode env is unset in production-mode resolution,
When service api TLS mode is resolved,
Then resolution fails closed unless valid TLS cert/key env inputs are provided.

### AC-2 Test Harness Compatibility
Given TLS mode env is unset in test-mode resolution,
When service api TLS mode is resolved,
Then mode resolves to `Disabled` for deterministic local test operation.

### AC-3 Explicit Mode Compatibility
Given explicit tls mode env is set to `disabled` or `require`,
When resolution occurs,
Then existing explicit mode semantics remain intact and deterministic.

## Conformance Cases

- C-01 (AC-1, Unit): unset mode + production simulation + missing cert/key yields deterministic error.
- C-02 (AC-2, Unit): unset mode + test simulation resolves `ServiceApiTlsMode::Disabled`.
- C-03 (AC-3, Unit): explicit disabled remains accepted; explicit require still validates cert/key inputs.

## Success Signals

1. Default production resolution no longer silently disables TLS.
2. Existing test harnesses continue to run without mandatory TLS fixtures.
3. TLS resolution tests cover default and explicit branches.
