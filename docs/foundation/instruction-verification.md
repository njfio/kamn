# Instruction Verification Pipeline (Issues #144, #145)

This document captures the first implementation slice of anti-hallucination instruction verification controls.

## Scope Delivered
- Added `crates/kamn-core/src/instruction_verify.rs` with:
  - `InstructionRecord` (on-chain source of truth)
  - `InstructionClaim` (claimed instruction presented to the agent)
  - `VerificationContext` (on-chain records, authorized senders, deterministic clock)
  - `InstructionVerifier::verify(...)`
  - Typed rejection reasons in `VerificationFailure`
- Added integration tests in `crates/kamn-core/tests/instruction_verification.rs`.

## Verification Checks
1. Instruction exists on-chain.
2. Claim sender matches on-chain sender.
3. Claim payload hash matches on-chain payload hash.
4. Claim signature matches on-chain signature.
5. Sender is explicitly authorized.
6. Claim is not expired relative to deterministic current time.
7. bounded claim validity window is enforced against context policy.
8. one-time claim consumption is enforced on replay-aware verification path.

## Rejection Outcomes
- `MissingInstruction`
- `SenderMismatch`
- `PayloadMismatch`
- `SignatureMismatch`
- `UnauthorizedSender`
- `Expired`
- `OverlongValidityWindow`
- `ReplayClaim`
- overlong validity window is rejected (`Regression: #409`).
- replayed claim is rejected (`Regression: #414`).

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test instruction_verification
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
