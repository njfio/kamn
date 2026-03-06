## Objective
Standardize the runtime request identity helper and directly related runtime test fixtures on the canonical DID shape `kamn:did:{role}:{id}`.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-kolme/src/runtime_request_identity_policy.rs`
  - `crates/kamn-kolme/tests/runtime_request_identity_policy_contracts.rs`
  - `crates/kamn-core/src/runtime_tests.rs`
  - `docs/architecture/did-format-standardization.md`
- Outputs:
  - canonical `kamn:did:{role}:{id}` examples in runtime request identity helper contracts
  - canonical `kamn:did:{role}:{id}` examples in matching Kolme/runtime tests

## Boundaries/Non-goals
- Do not change `AgentDid` or `KamnDid` parsing behavior.
- Do not introduce backward-compatibility aliases or dual-format support.
- Do not migrate unrelated docs or other crates in this issue.
- Do not change proposal-planner logic beyond canonical test fixture values.

## Failure modes
- Runtime request identity helpers still render `did:kamn:{role}:{id}` in idempotency keys or wire payloads.
- Kolme contract tests continue asserting divergent DID examples.
- Runtime proposal-planner fixtures continue using divergent DID strings.
- Focused tests pass only by weakening assertions rather than standardizing the canonical values.

## Acceptance criteria
- [ ] `deterministic_runtime_commit_idempotency_key` test coverage uses canonical `kamn:did:agent:*` examples.
- [ ] `render_runtime_commit_wire_payload` test coverage uses canonical `kamn:did:agent:*` examples.
- [ ] `crates/kamn-kolme/tests/runtime_request_identity_policy_contracts.rs` asserts canonical DID strings for request identity outputs.
- [ ] `crates/kamn-core/src/runtime_tests.rs` proposal-planner fixtures use canonical `kamn:did:agent:*` sender values.
- [ ] Focused test commands pass locally.

## Files to touch
- `crates/kamn-kolme/src/runtime_request_identity_policy.rs`
- `crates/kamn-kolme/tests/runtime_request_identity_policy_contracts.rs`
- `crates/kamn-core/src/runtime_tests.rs`
- `specs/6494-standardize-runtime-request-identity-dids.md`

## Error semantics
- No error behavior should change in this issue.
- Existing failure conditions and typed errors must remain unchanged.
- This issue only changes canonical example values and the resulting deterministic strings built from them.

## Test plan
- Red:
  - update focused tests to assert canonical DID strings and verify they fail before implementation
- Green:
  - `cargo test -p kamn-kolme runtime_request_identity_policy -- --nocapture`
  - `cargo test -p kamn-core runtime_tests -- --nocapture`
- Refactor:
  - rerun the focused commands after any duplication cleanup
