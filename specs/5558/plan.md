# Issue #5558 Plan - PRD Phase-1 kamn-agent-lib Foundation Implementation

## Approach
1. Create deterministic phase-1 baseline tests (RED) that assert required module presence and expected API contracts.
2. Scaffold `crates/kamn-agent-lib` with thin, typed wrappers over existing `kamn-sdk`/`kamn-kolme` primitives:
   - `identity`: DID/keyfile helpers.
   - `auth`: request nonce + signature/header builder.
   - `envelope`: canonical signed envelope constructor.
   - `client`: service route facade.
   - `kolme`: proof verification adapter.
   - `nonce`: monotonic nonce tracker.
   - `errors`: crate-specific error taxonomy.
   - `lib`: `KamnAgentHandle` orchestration API.
3. Implement GREEN behavior minimally to satisfy conformance cases.
4. Add docs/research phase-1 coverage report and PRD status notes.
5. Run quality gates and targeted regression checks.

## Affected Modules
- `Cargo.toml` (workspace member registration)
- `crates/kamn-agent-lib/Cargo.toml`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-agent-lib/src/identity.rs`
- `crates/kamn-agent-lib/src/auth.rs`
- `crates/kamn-agent-lib/src/envelope.rs`
- `crates/kamn-agent-lib/src/client.rs`
- `crates/kamn-agent-lib/src/kolme.rs`
- `crates/kamn-agent-lib/src/nonce.rs`
- `crates/kamn-agent-lib/src/errors.rs`
- `crates/kamn-agent-lib/tests/auth_roundtrip.rs`
- `crates/kamn-agent-lib/tests/envelope_construction.rs`
- `crates/kamn-agent-lib/tests/kolme_verification.rs`
- `docs/research/e2e-live-testing-prd-phase1-gap-analysis.md`
- `docs/prd/e2e-live-testing-prd.md`

## Risks and Mitigations
- Risk: mismatch between PRD route list and currently available Service API routes.
  - Mitigation: phase-1 wrappers target currently implemented routes and fail-closed with explicit unsupported-operation errors.
- Risk: auth/header format drift against service endpoint validation.
  - Mitigation: base auth logic on existing `kamn-sdk::service_signature_for_fields` and add integration tests.
- Risk: introducing redundant models already present in `kamn-sdk`.
  - Mitigation: keep agent-lib thin and compositional; avoid parallel protocol definitions.

## Interfaces / Contracts
- `KamnAgentHandle` contract:
  - identity bootstrap + metadata retrieval
  - message send/query
  - channel create/query/list messages
  - task create/query/accept/complete
  - escrow fund/release
  - health + proof verify
- `KamnAuthHeaders::build` deterministic header generation:
  - `x-kamn-sender-did`
  - `x-kamn-request-nonce`
  - `x-kamn-request-signature`
  - optional `x-kamn-authz-scope`

## ADR
- Not required for this phase-1 additive wrapper crate.
