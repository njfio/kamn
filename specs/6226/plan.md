# Issue 6226 Plan

## Approach
1. Add `kamn-types` crate and expose canonical DID types by re-exporting from `kamn-core` as the first unification step.
2. Update workspace member list and crate dependencies.
3. Remove local SDK `AgentDid` definition and import shared type from `kamn-types`.
4. Add `From<kamn_types::AgentDidError> for SdkError` mapping to preserve ergonomic `?` use and fail-closed behavior.
5. Add/adjust SDK tests for parse-error mapping contracts.

## Affected Modules
- `Cargo.toml`
- `crates/kamn-types/Cargo.toml`
- `crates/kamn-types/src/lib.rs`
- `crates/kamn-sdk/Cargo.toml`
- `crates/kamn-sdk/src/types.rs`
- `crates/kamn-sdk/src/error.rs`
- `crates/kamn-sdk` tests touching parse error behavior

## Risks and Mitigations
- Risk: tighter canonical DID parsing changes SDK behavior.
  - Mitigation: explicit SDK mapping tests and clear error normalization to `SdkError::InvalidInput`.
- Risk: transitive dependency overhead from `kamn-types -> kamn-core` in this first phase.
  - Mitigation: document as phase-1 compatibility step; full extraction tracked under core-extraction task.

## Interfaces
- Shared type export: `kamn_types::AgentDid`.
- SDK compatibility interface: `impl From<kamn_types::AgentDidError> for SdkError`.
