# 6912-split-data-layer-m9-realtime-delivery

## Objective
Split `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs` into bounded concern-based modules while preserving deterministic realtime-delivery behavior for presence visibility, channel dispatch authorization, recipient queue projection, dispatch acknowledgements, anti-spam admission, and runtime backpressure projection.

## Inputs/Outputs
- Input: current `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs` production source and the existing `kamn-core` behavior tests that exercise M9 realtime delivery contracts.
- Output: a thin root shell that delegates to bounded sibling modules for models/constants, presence and relationship flow, queue/dispatch flow, runtime backpressure projection, authorization/validation helpers, and tests.
- Output: a hard-fail extraction contract that enforces the root shell budget and extracted module layout.

## Boundaries/Non-goals
- Do not change M9 reason-code values, queue-cap thresholds, or deterministic escalation semantics.
- Do not change anti-spam admission semantics or channel membership authorization behavior.
- Do not change public field names or public return types.
- Do not add dependencies or weaken current tests to make the split pass.

## Failure modes
- The root file remains oversized while the extraction contract passes.
- Presence visibility, relationship-link normalization, or owner-scope authorization semantics drift during extraction.
- Dispatch acknowledgement, queue-depth, or escalation thresholds change silently.
- Runtime backpressure projection changes reason-code mapping or error translation semantics.
- Any touched extracted file or function fails the touched-Rust size policy.

## Acceptance criteria
- [ ] `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs` becomes a thin root shell under the active file-size policy.
- [ ] Bounded sibling modules exist for models/constants, presence/relationship flow, queue/dispatch flow, runtime backpressure projection, validation/authorization helpers, and tests.
- [ ] Existing realtime-delivery behavior remains unchanged after the split.
- [ ] A hard-fail extraction contract exists and passes.
- [ ] The real `data_layer_m9_realtime_delivery` behavior target still passes after the split.
- [ ] Touched-Rust size policy returns `GO` on the final branch.

## Files to touch
- `crates/kamn-core/src/data_layer_m9_realtime_delivery.rs`
- `crates/kamn-core/src/data_layer_m9_realtime_delivery/`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery_module_extraction_contract.rs`
- `specs/6912-split-data-layer-m9-realtime-delivery.md`

## Error semantics
- Existing fail-closed M9 validation, anti-spam, and runtime backpressure errors remain typed and externally observable.
- No new fallbacks, silent coercions, or swallowed authorization/projection failures are introduced.
- Invalid DIDs, owner-scope mismatches, membership failures, and backpressure-policy errors remain deterministic and explicit.

## Test plan
- Add a red extraction contract that fails while the root file remains oversized and the expected module layout is missing.
- Run the extraction contract target and confirm red.
- Run the realtime-delivery behavior target after extraction.
- Run touched-Rust size policy on the final branch.

## Final evidence
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery_module_extraction_contract -- --nocapture`
- `cargo test -p kamn-core --test data_layer_m9_realtime_delivery -- --nocapture`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn --base-ref origin/main --output-json /tmp/6912-touched-size-refactor4.json`
- touched-Rust result: `policy_decision=GO`

## Integration verification
- The root shell remains the real `kamn-core` production entrypoint for M9 realtime delivery.
- Presence, dispatch, queue projection, anti-spam, and runtime backpressure behavior remain exercised through the existing `data_layer_m9_realtime_delivery` integration-style target.
- No public API surface or reason-code contract changed during extraction.

## Deviations
- None.
