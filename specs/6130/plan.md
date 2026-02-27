# Plan: Issue #6130

## Approach
1. Add RED regression expectation updates in `crates/kamn-agent-lib/src/identity.rs` to reflect explicit FNV-1a semantics.
2. Refactor `derive_name_seed_bytes` to use a dedicated FNV-1a round helper (`hash = (hash ^ byte) * prime`) across source and index-salt bytes.
3. Keep scalar non-zero guard (`output[0] |= 0x01`) unchanged.
4. Run scoped `kamn-agent-lib` tests, then fmt/clippy verification.

## Affected Modules
- `crates/kamn-agent-lib/src/identity.rs`

## Risks
- Risk: deterministic signing-key vectors change and can affect tests expecting hardcoded values.
  - Mitigation: update known-vector tests and run full `kamn-agent-lib` test suite.
- Risk: subtle behavior drift in identity consumers.
  - Mitigation: run crate-level tests covering envelope/auth flows.

## Interfaces/Contracts
- Public `AgentIdentity::from_agent_name` remains unchanged.
- Deterministic output vector for specific names may change due to corrected derivation semantics.
