# Plan: Issue #5899 - Immediate Security/Runtime Remediation (Production Blockers)

## Approach
1. Remove hardcoded fallback key constants and centralize signer key resolution to explicit environment/provider inputs.
2. Refactor transaction signing path to fail closed on signer failures; delete deterministic fallback path from failure branch.
3. Normalize signature taxonomy constants to accurately represent cryptographic vs deterministic profiles.
4. Replace targeted string-scanning JSON field extraction in `kamn-mcp-server` and `kamn-sdk` touched request parsing helpers with `serde_json::Value` extraction and robust error mapping.
5. Add bounded-capacity replay/state guard behavior in touched maps and cover with deterministic regression tests.

## Affected Modules
- `crates/kamn-core/src/signer_backend.rs`
- `crates/kamn-core/src/transaction.rs`
- `crates/kamn-core/src/signature_profile.rs`
- `crates/kamn-mcp-server/src/protocol.rs`
- `crates/kamn-mcp-server/src/dispatch.rs`
- `crates/kamn-sdk/src/service.rs`
- (tests in corresponding crates)

## Risks and Mitigations
- Risk: Removing fallback key/fallback signatures can break existing tests relying on permissive behavior.
  - Mitigation: update tests to assert explicit error semantics and provide fixture env values where required.
- Risk: JSON parser replacement can alter error text and compatibility.
  - Mitigation: preserve reason codes/error classes while changing parsing internals.
- Risk: Bounded replay guards can evict state too aggressively.
  - Mitigation: choose deterministic caps and add regression tests for acceptance window behavior.

## Interfaces / Contracts
- No wire-format schema change in this slice.
- Error behavior changes from fallback success to explicit failure for signer failure paths.
- JSON parsing helpers move from substring extraction to structured parse semantics.

## ADR
- Not required for this targeted remediation slice.
