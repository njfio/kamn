# Plan: Issue #5977

## Approach
- Introduce/route cryptographic signing helper in SDK request builder.
- Update agent-lib nonce/signature envelope builder to use same profile.
- Align service verifier expectations and tests.

## Affected Modules
- `crates/kamn-sdk/src/tcp.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `crates/kamn-sdk/tests/*` and `crates/kamn-agent-lib/tests/*`

## Risks / Mitigations
- Fixture drift in tests expecting baseline strings.
  Mitigation: migrate fixtures to cryptographic expectations and keep explicit legacy compatibility tests where needed.

## Interfaces / Contracts
- Request signature header schema and verification semantics.
