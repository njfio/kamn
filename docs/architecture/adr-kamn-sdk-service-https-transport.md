# ADR: kamn-sdk Service API HTTPS Transport

- Status: Accepted
- Date: 2026-02-25
- Issue: #5930
- Spec: `specs/5930/spec.md`

## Context

`kamn-sdk` parsed `https://` service endpoints but returned `SdkError::NotImplemented` for all requests. This forced plaintext transport and blocked production-grade service API connectivity. The issue scope requires strict certificate validation, deterministic fail-closed behavior for TLS misconfiguration and invalid chains, and integration coverage using real fixture certificates.

## Decision

1. Implement HTTPS transport in `kamn-sdk` service client using `rustls` (`ClientConnection` + `StreamOwned`) over `TcpStream`.
2. Keep strict trust validation as default behavior. Certificates are validated against:
   - built-in `webpki-roots` trust roots by default, or
   - an explicit custom CA bundle path from `KAMN_SERVICE_API_TLS_CA_FILE`.
3. Map TLS failures into deterministic SDK errors:
   - certificate validation failures -> `SdkError::TransportFailure("service tls certificate verification failed")`
   - CA file read/parse/missing-cert conditions -> deterministic transport failures
   - malformed custom CA env input -> `SdkError::InvalidInput`.
4. Add fixture-backed HTTPS integration tests in `crates/kamn-sdk/tests/service_api_client.rs` to enforce conformance.

## Consequences

- `kamn-sdk` now supports `https://` endpoints in production request paths.
- The crate gains direct TLS dependencies: `rustls`, `rustls-pemfile`, `webpki-roots`.
- TLS trust behavior is explicit and test-covered, reducing risk of silent insecure fallback.
- Environment configuration now includes one SDK client trust override: `KAMN_SERVICE_API_TLS_CA_FILE`.
