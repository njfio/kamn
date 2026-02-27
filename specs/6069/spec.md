# Spec: Issue #6069 - Harden Service API TLS default for non-loopback binds

- Issue: #6069
- Status: Reviewed
- Type: task
- Priority: P1
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-26
- Parent: #6070

## Problem Statement
Service API currently defaults to plaintext (`disabled`) when TLS mode env is absent, regardless of bind address. For network-exposed deployments this creates an insecure-by-default transport posture.

## Scope
In scope:
- Make missing TLS mode env resolve to `disabled` only for loopback bind addresses.
- Fail startup when TLS mode env is missing for non-loopback bind addresses.
- Keep explicit TLS mode values (`disabled`, `require`) behavior intact.
- Add resolution tests for loopback/non-loopback branches.

Out of scope:
- certificate provisioning workflow.
- observability endpoint TLS policy.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `resolve_service_api_tls_mode` returns `Disabled` when env is missing and bind address is loopback.
- AC-2: `resolve_service_api_tls_mode` returns an explicit configuration error when env is missing and bind address is non-loopback.
- AC-3: Explicit env values (`disabled` and `require` with valid files) continue to work.

## Conformance Cases
- C-01 (Unit, AC-1): missing env + `127.0.0.1:PORT` resolves to `Disabled`.
- C-02 (Unit, AC-2): missing env + `0.0.0.0:PORT` returns error containing TLS mode env guidance.
- C-03 (Regression, AC-3): existing explicit env-path tests continue to pass.

## Success Metrics / Observable Signals
- RED test fails before bind-aware default resolution exists.
- GREEN tests pass for loopback/non-loopback resolution behavior.
- `cargo test -p kamn-node service_api_state_file_resolution` and new TLS tests pass.
