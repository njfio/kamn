# 6616-add-runtime-guards-anti-spam-coverage

## Objective
Add dedicated crate-level integration coverage for `kamn-runtime-guards` anti-spam behavior through the public boundary so deposit admission, duplicate-message rejection, rolling-window rate limiting, suspension, and input/config failure modes are verified outside the inline unit tests.

## Inputs/Outputs
Inputs:
- `AntiSpamConfig`
- `AntiSpamEngine`
- `sender_did`, `message_id`, and `now_unix` inputs passed through the public anti-spam API

Outputs:
- `Result<AntiSpamDecision, AntiSpamError>`
- `AntiSpamTelemetry` snapshots after behavior transitions

## Boundaries/Non-goals
- No changes to anti-spam production policy semantics unless a real defect is exposed by red tests
- No changes to other runtime-guards modules (`watchdog`, `quota_policy`, `fairness_policy`, etc.)
- No new dependencies
- No public API additions

## Failure modes
- invalid config returns `AntiSpamError::InvalidConfig`
- invalid sender DID returns `AntiSpamError::InvalidInput`
- empty message id returns `AntiSpamError::InvalidInput`
- sender without required deposit returns `AntiSpamDecision::Rejected(AntiSpamRejection::InsufficientDeposit { .. })`
- duplicate message id returns `AntiSpamDecision::Rejected(AntiSpamRejection::DuplicateMessageId(..))`
- sender exceeding the rolling window returns `AntiSpamDecision::Rejected(AntiSpamRejection::RateLimitExceeded { .. })`
- sender exceeding the suspension threshold remains blocked until `suspended_until_unix`

## Acceptance criteria
- [ ] a dedicated integration test file exists under `crates/kamn-runtime-guards/tests/`
- [ ] the public anti-spam API accepts a first message from a funded sender and records accepted telemetry
- [ ] the public anti-spam API rejects duplicate message ids and increments duplicate telemetry
- [ ] the public anti-spam API rejects unfunded senders with typed insufficient-deposit rejection
- [ ] the public anti-spam API rejects over-window senders with typed rate-limit rejection and then suspends on the configured threshold
- [ ] the public anti-spam API allows a sender again after suspension expiry and a cleared rate window
- [ ] invalid config and invalid input paths are covered through the public boundary
- [ ] `cargo test -p kamn-runtime-guards -- --nocapture` passes

## Files to touch
- `specs/6616-add-runtime-guards-anti-spam-coverage.md`
- `crates/kamn-runtime-guards/tests/runtime_guard_anti_spam.rs`
- `crates/kamn-runtime-guards/tests/runtime_guard_anti_spam_contract.rs` if a source-contract harness is needed to pin the dedicated test surface
- `fixtures/ci/test_file_size_policy_baseline.env` only if the new test target changes the workspace inventory baseline

## Error semantics
- tests must assert exact typed error variants and rejection variants rather than string matching where the public API already exposes structured errors
- this issue must fail closed: any invalid input/config path should return a typed error or rejection, never a silent accept path

## Test plan
1. Add red integration tests for funded acceptance, insufficient deposit rejection, duplicate rejection, rate-limit rejection, suspension, and post-suspension recovery.
2. Add red integration tests for invalid config and invalid input error paths.
3. Implement the minimum changes required to make the red tests pass. Prefer zero production changes if the public API already satisfies the contract.
4. Refactor tests for duplication and clarity while keeping file/function sizes within repo policy.
5. Run targeted and full crate tests:
   - `cargo test -p kamn-runtime-guards --test runtime_guard_anti_spam -- --nocapture`
   - `cargo test -p kamn-runtime-guards -- --nocapture`
