# Plan: Issue #6131

## Approach
1. Add RED tests in `crates/kamn-runtime-guards/src/anti_spam.rs` for bounded retention and eviction behavior.
2. Extend `AntiSpamConfig` with `max_seen_message_ids` and validate it in `validate_config`.
3. Implement bounded duplicate tracking in `AntiSpamEngine` using insertion-order tracking plus set membership.
4. Update explicit `AntiSpamConfig` struct literals in downstream tests affected by the new field.
5. Run scoped fmt/clippy/tests for `kamn-runtime-guards` and impacted `kamn-core` anti-spam tests.

## Affected Modules
- `crates/kamn-runtime-guards/src/anti_spam.rs`
- `crates/kamn-core/tests/anti_spam_controls.rs`
- `crates/kamn-core/tests/data_layer_m9_realtime_delivery.rs`

## Risks
- Risk: eviction semantics could unexpectedly permit old duplicate IDs.
  - Mitigation: codify intended FIFO eviction with explicit conformance tests.
- Risk: config-surface change can break explicit struct literals.
  - Mitigation: update all literal initializers and run impacted tests.

## Interfaces/Contracts
- `AntiSpamConfig` gains `max_seen_message_ids: usize`.
- Duplicate-message protection contract becomes bounded-retention instead of unbounded lifetime retention.
