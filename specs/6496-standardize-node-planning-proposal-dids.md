## Objective
Standardize the remaining `kamn-node` planning-mode proposal fixtures and command examples on the canonical DID shape `kamn:did:{role}:{id}`.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-node/src/main_tests/runtime_tests/runtime_mode_and_transport_profile_tests.rs`
  - `crates/kamn-node/src/main_tests/cli_contract_tests.rs`
  - `crates/kamn-node/src/main_tests/report_tests.rs`
  - `docs/foundation/node-runtime-cli.md`
  - `docs/architecture/did-format-standardization.md`
- Outputs:
  - canonical `kamn:did:agent:*` proposal fixtures in node runtime/CLI tests
  - canonical planning-mode DID examples in the node runtime CLI doc
  - refreshed DID divergence inventory that no longer points at the cleaned node fixtures

## Boundaries/Non-goals
- Do not change parser behavior or add dual-format support.
- Do not migrate unrelated remaining docs outside the listed node surfaces.
- Do not change runtime planner logic or error semantics.

## Failure modes
- Node planning-mode test fixtures still use `did:kamn:agent:*`.
- The node runtime CLI doc still shows divergent planning-mode proposal examples.
- The DID divergence inventory still lists the cleaned node files as active divergent consumers.
- Focused tests pass only because assertions were weakened rather than because fixtures were standardized.

## Acceptance criteria
- [ ] `crates/kamn-node/src/main_tests/runtime_tests/runtime_mode_and_transport_profile_tests.rs` uses canonical `kamn:did:agent:*` proposal fixtures.
- [ ] `crates/kamn-node/src/main_tests/cli_contract_tests.rs` uses canonical `kamn:did:agent:*` proposal fixtures.
- [ ] `crates/kamn-node/src/main_tests/report_tests.rs` uses canonical `kamn:did:agent:*` proposal fixtures where planning proposals are asserted.
- [ ] `docs/foundation/node-runtime-cli.md` uses canonical `kamn:did:agent:*` planning-mode examples.
- [ ] `docs/architecture/did-format-standardization.md` no longer lists the cleaned node fixtures as current divergent consumers.
- [ ] Focused test/doc contract commands pass locally.

## Files to touch
- `crates/kamn-node/src/main_tests/runtime_tests/runtime_mode_and_transport_profile_tests.rs`
- `crates/kamn-node/src/main_tests/cli_contract_tests.rs`
- `crates/kamn-node/src/main_tests/report_tests.rs`
- `docs/foundation/node-runtime-cli.md`
- `docs/architecture/did-format-standardization.md`
- `crates/kamn-types/tests/identity_boundary_contract.rs`
- `specs/6496-standardize-node-planning-proposal-dids.md`

## Error semantics
- No error behavior should change in this issue.
- Existing planning-mode argument validation and runtime planner errors must remain unchanged.
- This issue changes fixture/example values and resulting deterministic strings only.

## Test plan
- Red:
  - update focused node/doc fixtures to canonical `kamn:did:...` values and verify whether existing parsing logic already accepts them
- Green:
  - `cargo test -p kamn-node runtime_mode_and_transport_profile_tests -- --nocapture`
  - `cargo test -p kamn-node cli_contract_tests -- --nocapture`
  - `cargo test -p kamn-node report_tests -- --nocapture`
  - `cargo test -p kamn-types --test identity_boundary_contract -- --nocapture`
- Refactor:
  - rerun the focused commands after inventory/doc cleanup
