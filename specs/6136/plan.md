# Plan: Issue #6136

## Approach
1. Add a dedicated subsection to `docs/foundation/kolme-runtime-commit-client.md` under deterministic request identity rules.
2. Include explicit statement of length-based compatibility behavior, rationale, and migration caveat.
3. Add a docs conformance test in `crates/kamn-kolme/tests/runtime_request_identity_policy_contracts.rs` checking required markers.
4. Run scoped `kamn-kolme` tests.

## Affected Modules
- `docs/foundation/kolme-runtime-commit-client.md`
- `crates/kamn-kolme/tests/runtime_request_identity_policy_contracts.rs`

## Risks
- Risk: wording drift can reintroduce ambiguity.
  - Mitigation: enforce stable marker test over required wording anchors.

## Interfaces/Contracts
- Documentation and test-only change; no runtime behavior changes.
