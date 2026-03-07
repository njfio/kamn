# 6570 Cache Group Master Seed Lazily

## Objective
Stop reparsing `KAMN_KEY_AGREEMENT_MASTER_SEED_HEX` on every group-message crypto operation while preserving the current constructor contract, and zeroize the env-loaded seed hex buffer after parsing in both group and direct-message crypto.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-core/src/group_channel_crypto.rs`
  - `crates/kamn-crypto/src/direct_message_crypto.rs`
- Outputs:
  - lazy in-engine cache for parsed group master seed material
  - zeroized env-loaded hex string buffers after direct/group parse
  - regression coverage for lazy caching and zeroization source markers

## Boundaries/Non-goals
- Do not change public ciphertext formats or algorithm labels.
- Do not require `GroupChannelCryptoEngine::new()` to fail when the env seed is absent.
- Do not redesign key derivation or remove env-based seed sourcing.
- Do not add new dependencies.

## Failure modes
- group crypto still reparses the env seed on every `encrypt()` or `decrypt()` call.
- group engine constructor starts requiring the env seed.
- env-loaded seed hex strings remain live without explicit zeroization after parsing.
- missing or invalid env seed values stop failing with the existing typed errors.
- tests miss a regression back to per-operation env parsing.

## Acceptance criteria
- [x] `GroupChannelCryptoEngine` caches parsed master seed material after the first operation that needs it.
- [x] `GroupChannelCryptoEngine::new()` still succeeds without requiring `KAMN_KEY_AGREEMENT_MASTER_SEED_HEX`.
- [x] repeated group encrypt/decrypt operations do not need repeated env reparsing once the seed is cached.
- [x] direct-message seed loading zeroizes the env-loaded hex buffer after parsing.
- [x] group-message seed loading zeroizes the env-loaded hex buffer after parsing.
- [x] missing and invalid env seed values still fail closed with the existing typed errors.
- [x] regression tests fail if the lazy cache or zeroization markers disappear.

## Files to touch
- `crates/kamn-core/src/group_channel_crypto.rs`
- `crates/kamn-core/tests/group_sender_keys.rs`
- `crates/kamn-crypto/src/direct_message_crypto.rs`
- `specs/6570-cache-group-master-seed-lazily.md`

## Error semantics
- group crypto continues returning `MissingKeyAgreementMasterSeed` and `InvalidKeyAgreementMasterSeed` when the env source is unavailable or malformed.
- direct-message crypto continues returning the same typed seed errors at engine construction time.
- cache initialization must fail closed; there are no silent fallbacks or default seeds.

## Test plan
- Red:
  - add group tests that require constructor success without env seed but require a single cached load after env availability is restored
  - add source-contract tests for zeroization markers in direct/group seed loaders
- Green:
  - add the minimal lazy cache and zeroization logic to satisfy the tests
- Integration:
  - rerun targeted direct/group crypto tests and strict clippy

## Deviations
- None.

## Verification evidence
- `cargo test -p kamn-core group_channel_crypto -- --nocapture`
- `cargo test -p kamn-crypto direct_message_crypto -- --nocapture`
- `cargo test -p kamn-core --test group_sender_keys -- --nocapture`
- `cargo test -p kamn-crypto --test direct_message_crypto_integration -- --nocapture`
- `cargo clippy -p kamn-core --tests -- -D warnings`
- `cargo clippy -p kamn-crypto --tests -- -D warnings`
- `bash scripts/ci/run_critical_path_coverage_gate.sh --threshold-file .ci/critical-path-coverage-thresholds.json --core-json /tmp/ci-critical-path-core-6570b.json --node-json /tmp/ci-critical-path-node-6570b.json --output-json /tmp/ci-critical-path-policy-6570b.json`

## Shell-surface closure template
- shell_loc_delta_actual: 0
- rust_loc_delta_actual: 174
- shell_to_rust_ratio_delta_actual: 0.0
- shell_surface_ratio_target_status: improved
