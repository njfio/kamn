# Plan: Issue #6073

## Approach
1. Add RED unit tests in `auth.rs` for auth key selection behavior.
2. Add runtime state support for optional DID->public-key map.
3. Parse DID key map from env in `server.rs` and validate shape.
4. Update auth verification path to use sender-specific key when map exists.
5. Verify with targeted tests, fmt, and clippy.

## Affected Modules
- `crates/kamn-node/src/service_api_endpoint.rs`
- `crates/kamn-node/src/service_api_endpoint/server.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`
- `specs/6073/spec.md`
- `specs/6073/plan.md`
- `specs/6073/tasks.md`

## Risks / Mitigations
- Risk: malformed DID-key map env breaks startup.
  Mitigation: strict parse/validation with explicit error messages.
- Risk: compatibility break for deployments relying on shared key only.
  Mitigation: retain single-key fallback when map env is absent.

## Interfaces / Contracts
- New env contract: optional JSON object mapping sender DID string to auth public key hex.
- Auth contract: configured map takes precedence over shared key.
