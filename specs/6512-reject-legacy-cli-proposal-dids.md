## Objective
Reject legacy `did:kamn:...` proposal sender DIDs in the node CLI planning parser so the CLI
matches the documented fail-closed DID policy at parser boundaries.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-node/src/cli_value_parsers.rs`
  - `crates/kamn-node/src/main_tests/cli_contract_tests.rs`
  - `crates/kamn-node/src/main_tests/runtime_tests/runtime_mode_and_transport_profile_tests.rs`
  - `docs/architecture/did-format-standardization.md`
- Outputs:
  - fail-closed rejection of legacy proposal sender DIDs in `parse_proposal_candidate()`
  - preserved acceptance of canonical `kamn:did:agent:*` proposal sender DIDs
  - focused regression coverage for the CLI parser and runtime-mode parsing path

## Boundaries/Non-goals
- Do not change `parse_rejoin_attempt()` or `node_id` semantics.
- Do not add compatibility rewrites from `did:kamn:...` to `kamn:did:...`.
- Do not change runtime proposal ordering, reporting, or planner output formatting.
- Do not modify docs unless test markers or rollout notes require a minimal inventory update.

## Failure modes
- `parse_proposal_candidate()` still accepts `did:kamn:agent:*` sender DIDs.
- Canonical `kamn:did:agent:*` proposal sender DIDs regress and stop parsing.
- Legacy DID rejection is surfaced through a different error family than the existing malformed
  proposal argument path.
- Runtime-mode planning tests pass only because assertions were weakened instead of the parser
  enforcing the intended boundary.

## Acceptance criteria
- [x] `parse_proposal_candidate()` rejects legacy `did:kamn:agent:*` sender DIDs.
- [x] `parse_proposal_candidate()` still accepts canonical `kamn:did:agent:*` sender DIDs.
- [x] Rejected legacy proposal sender DIDs surface through the existing
      `ConfigError::InvalidProposalArgument` CLI path with no silent rewrite.
- [x] Focused CLI/runtime tests cover both the reject and accept paths.

## Files to touch
- `crates/kamn-node/src/cli_value_parsers.rs`
- `crates/kamn-node/src/main_tests/cli_contract_tests.rs`
- `crates/kamn-node/src/main_tests/runtime_tests/runtime_mode_and_transport_profile_tests.rs`
- `specs/6512-reject-legacy-cli-proposal-dids.md`

## Error semantics
- Legacy `did:kamn:...` proposal sender DIDs must fail closed at the CLI parser boundary.
- The failure must remain observable through `ConfigError::InvalidProposalArgument`.
- No fallback, normalization, or dual-format acceptance is allowed.

## Test plan
- Red:
  - add a CLI contract test that expects legacy proposal sender DIDs to fail
  - add a runtime-mode parsing test that expects legacy proposal sender DIDs to fail
  - confirm canonical `kamn:did:agent:*` proposal sender DIDs still parse
- Green:
  - `cargo test -p kamn-node rejects_legacy_proposal_sender_did_argument -- --nocapture`
  - `cargo test -p kamn-node rejects_runtime_mode_planning_with_legacy_proposal_sender_did -- --nocapture`
  - `cargo test -p kamn-node parses_runtime_mode_planning_with_proposals -- --nocapture`
- Refactor:
  - rerun the focused `kamn-node` parser/runtime tests after cleanup

## Deviations
- None.

## Execution Evidence
- Red:
  - `cargo test -p kamn-node rejects_legacy_proposal_sender_did_argument -- --nocapture`
  - `cargo test -p kamn-node rejects_runtime_mode_planning_with_legacy_proposal_sender_did -- --nocapture`
- Green:
  - `cargo test -p kamn-node rejects_legacy_proposal_sender_did_argument -- --nocapture`
  - `cargo test -p kamn-node rejects_runtime_mode_planning_with_legacy_proposal_sender_did -- --nocapture`
  - `cargo test -p kamn-node parses_runtime_mode_planning_with_proposals -- --nocapture`
- Refactor / Integration:
  - `cargo test -p kamn-node cli_contract_tests -- --nocapture`
  - `cargo test -p kamn-node runtime_mode_and_transport_profile_tests -- --nocapture`
  - `cargo clippy -p kamn-node --tests -- -D warnings`
