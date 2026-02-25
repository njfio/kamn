# Plan: Issue #5974

## Approach
- Migrate `kamn-sdk`/`kamn-agent-lib` request signing to cryptographic profile helpers already present in core signer contracts.
- Update service-side profile validation expectations and compatibility switches.
- Add/adjust integration tests for request signing/verifying/tamper/replay/wrong-key behaviors.

## Affected Modules (Expected)
- `crates/kamn-sdk/src/tcp.rs`
- `crates/kamn-agent-lib/src/lib.rs`
- `crates/kamn-core/src/signature_profile.rs`
- `crates/kamn-node/src/service_api_endpoint/auth.rs`

## Risks / Mitigations
- Compatibility regressions for local fixtures.
  Mitigation: keep explicit opt-in compatibility path for deterministic fixtures.

## Interfaces / Contracts
- Service API auth signature header contract.
- Signature profile parsing and fail-closed validation contract.
