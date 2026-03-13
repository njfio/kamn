# 6952-split-signature-profile

## Objective
Reduce `crates/kamn-core/src/signature_profile.rs` from its current monolithic form into a bounded root shell plus concern-based submodules without changing signature-profile behavior.

## Inputs/Outputs
- Inputs:
  - Existing signature profile metadata parsing, compatibility fixtures, service-auth signing helpers, and tests in `crates/kamn-core/src/signature_profile.rs`
  - Existing public callers through `kamn-core`
- Outputs:
  - Thin root shell at `crates/kamn-core/src/signature_profile.rs`
  - Extracted module tree under `crates/kamn-core/src/signature_profile/`
  - Hard-fail extraction contract covering root shell budget and expected module markers

## Boundaries/Non-goals
- No changes to signing, verification, compatibility, or metadata semantics
- No public API redesign beyond internal re-export wiring required by the split
- No unrelated cleanup outside `signature_profile.rs` and its extraction contract

## Failure modes
- Missing extracted module files
- Root shell still contains moved logic or tests inline
- Root shell exceeds staged line budget
- Split breaks public imports or service-auth helpers
- Split introduces touched-Rust size regressions in new files or functions

## Acceptance criteria
- [ ] `crates/kamn-core/src/signature_profile.rs` is reduced to a bounded root shell
- [ ] Extracted modules under `crates/kamn-core/src/signature_profile/` stay within the active size policy
- [ ] Existing signature-profile behavior remains green under real tests/checks
- [ ] `crates/kamn-core/tests/signature_profile_module_extraction_contract.rs` hard-fails on layout regressions
- [ ] Touched-Rust size policy returns `policy_decision=GO`

## Files to touch
- `crates/kamn-core/src/signature_profile.rs`
- `crates/kamn-core/src/signature_profile/*.rs`
- `crates/kamn-core/src/signature_profile/tests/*.rs`
- `crates/kamn-core/tests/signature_profile_module_extraction_contract.rs`
- `specs/6952-split-signature-profile.md`

## Error semantics
- Preserve existing `Result`-based and typed-error behavior
- Preserve current `ServiceAuthSignatureError` semantics and message context
- No silent fallback or swallowed parse/signature failures

## Test plan
1. Add a red extraction contract that fails while the root file remains monolithic
2. Extract the module tree and re-run the extraction contract to green
3. Run issue-local behavior checks for signature-profile compilation and tests
4. Run touched-Rust size policy against the issue write set


## Final evidence
- `cargo test -p kamn-core --test signature_profile_module_extraction_contract -- --nocapture`
- `cargo check -p kamn-core --lib`
- `cargo test -p kamn-core signature_profile::tests:: --lib -- --nocapture`
- `cargo test -p kamn-node --no-run`
- `cargo test -p kamn-sdk --no-run`
- `python3 scripts/ci/check_touched_rust_size_policy.py --repo-root /home/n/Code/kamn-active-20260313-093857 --base-ref origin/main --output-json /tmp/6952-touched-size.json`
- touched-Rust result: `policy_decision=GO`

## Deviations
- `cargo test -p kamn-core signature_profile::tests:: --lib -- --nocapture` is currently blocked on an unrelated current-main parse error in `crates/kamn-core/src/data_layer_m7_timeseries_telemetry/tests.rs` (`unexpected closing delimiter`). The split was verified with the extraction contract, `cargo check -p kamn-core --lib`, and downstream `--no-run` checks instead.
