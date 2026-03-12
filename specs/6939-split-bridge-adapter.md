# 6939-split-bridge-adapter

## Objective
Split `crates/kamn-core/src/bridge_adapter.rs` into bounded, concern-based modules while preserving bridge platform normalization, outbound translation, policy enforcement, replay protection, envelope conversion, and existing typed error behavior.

## Inputs/Outputs
- Inputs:
  - inbound bridge envelopes
  - outbound bridge requests
  - bridge policy decisions
  - canonical envelope conversion parameters
- Outputs:
  - unchanged bridge adapter behavior
  - a thin root shell in `bridge_adapter.rs`
  - bounded sibling modules for models, engine, errors, validation/support, and tests
  - a hard-fail extraction contract for the root shell and module layout

## Boundaries/Non-goals
- No changes to bridge runtime semantics, replay rules, or freshness behavior
- No changes to envelope field semantics or proof generation behavior
- No new dependencies
- No unrelated refactors outside the `bridge_adapter` surface

## Failure modes
- invalid bridge DID remains fail-closed
- invalid target or sender DIDs remain fail-closed
- stale inbound messages remain fail-closed
- duplicate inbound and outbound IDs remain fail-closed
- outbound request-id mutation remains fail-closed
- invalid canonical envelope conversion inputs remain fail-closed
- extraction contract fails if the root shell or module layout regress

## Acceptance criteria
- [x] `crates/kamn-core/src/bridge_adapter.rs` becomes a thin root shell under the active file-size budget
- [x] bounded modules separate models, engine behavior, typed errors, validation/support helpers, and tests
- [x] a hard-fail extraction contract enforces the root shell and module layout
- [x] existing bridge adapter tests remain green without semantic drift
- [x] touched-Rust size policy returns `policy_decision=GO`
- [x] final spec records test evidence and any deviations

## Files to touch
- `crates/kamn-core/src/bridge_adapter.rs`
- `crates/kamn-core/src/bridge_adapter/`
- `crates/kamn-core/tests/bridge_adapter_module_extraction_contract.rs`
- `specs/6939-split-bridge-adapter.md`

## Error semantics
- Preserve existing typed error behavior and stable reason markers
- Preserve fail-closed validation for bridge DIDs, timestamps, request IDs, and canonical envelope conversion
- Do not introduce silent fallbacks or relaxed policy behavior

## Test plan
- Add a red extraction contract that fails while `bridge_adapter.rs` remains monolithic
- Run the extraction contract green once the split is in place
- Run the real bridge adapter tests after extraction
- Run touched-Rust size policy against the staged write set

## Final evidence
- `cargo test -p kamn-core --test bridge_adapter_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core bridge_adapter::tests:: --lib -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-clean-20260312-101857-auth --base-ref github/main --output-json /tmp/6939-touched-size.json`
- Touched-Rust result: `policy_decision=GO`

## Deviations
- None.
