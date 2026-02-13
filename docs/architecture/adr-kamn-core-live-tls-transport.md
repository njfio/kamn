# ADR: `kamn-core` Live TLS Transport Dependency Posture

## Status

Accepted (2026-02-13)

## Context

`kamn-core` owns the live runtime-commit HTTP transport used by `kamn-node` when running Kolme live mode. That path must:

- submit signed runtime-commit payloads over HTTPS,
- poll finality endpoints over HTTPS,
- classify transport and TLS failures deterministically into fail-closed provider errors.

The previous "dependency-free transport" narrative no longer reflects implementation reality. The live HTTPS path is implemented in-process via:

- `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs`,
- trust policy contracts from `crates/kamn-kolme/src/tls_policy.rs`,
- transport-facing integration coverage in `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs`.

## Decision

Keep these dependencies in `kamn-core` for live HTTPS transport:

- `rustls`
- `rustls-pemfile`
- `webpki-roots`

Enforce the following constraints:

1. TLS execution remains in-process and deterministic.
2. Subprocess TLS paths (`curl`, `openssl s_client`) are not allowed in runtime transport code paths.
3. Optional custom trust roots are loaded through `KAMN_KOLME_TLS_CA_FILE` when provided.
4. Local deterministic/in-memory flows may run without network TLS at runtime.

## Alternatives Considered

### Subprocess `curl` or `openssl s_client`

Rejected. This increases operational variance across host images, adds process-spawn overhead, and weakens deterministic error taxonomy control.

### `reqwest` / async client migration

Deferred. This increases dependency and runtime surface area and would force broader runtime model changes unrelated to the current live-transport hardening scope.

### Compile-time feature gate for local-only builds

Accepted as follow-up candidate, not part of this ADR's baseline decision. Track under `#2756`.

## Consequences

Positive:

- Secure live HTTPS path with deterministic failure classification.
- No external TLS subprocess dependency in production runtime paths.
- Clear policy boundary between live mode and in-memory/local deterministic flows.

Tradeoffs:

- `kamn-core` is not dependency-minimal; TLS crates remain part of default build profile.
- Additional governance is needed to keep docs/tests aligned with this decision.

## Validation and Traceability

- Transport implementation: `crates/kamn-core/src/kolme_runtime_commit/http_transport.rs`
- TLS policy helpers: `crates/kamn-kolme/src/tls_policy.rs`
- Transport integration tests: `crates/kamn-core/tests/kolme_runtime_commit_http_transport.rs`
- Transport docs contract: `crates/kamn-core/tests/kolme_runtime_commit_client_docs.rs`

