# Instruction Verification Pipeline (Issues #144, #145, #553)

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
2. sender DID format validation: claim sender DID and on-chain sender DID are syntactically valid.
3. Claim sender matches on-chain sender.
4. Claim payload hash matches on-chain payload hash.
5. Claim and on-chain signatures must be non-empty.
6. Claim signature matches on-chain signature.
7. Sender is explicitly authorized.
8. Claim is not expired relative to deterministic current time.
9. bounded claim validity window is enforced against context policy.
10. one-time claim consumption is enforced on replay-aware verification path.
11. inclusion proof reference must be present and match the on-chain record.

## Rejection Outcomes
- `MissingInstruction`
- `SenderMismatch`
- `PayloadMismatch`
- `MissingClaimSignature`
- `MissingRecordSignature`
- `SignatureMismatch`
- `InvalidClaimSenderDid`
- `InvalidRecordSenderDid`
- `UnauthorizedSender`
- `Expired`
- `OverlongValidityWindow`
- `ReplayClaim`
- `MissingInclusionProofReference`
- `InclusionProofMismatch`
- overlong validity window is rejected (`Regression: #409`).
- replayed claim is rejected (`Regression: #414`).
- mismatched or missing inclusion proof reference is rejected (`Regression: #448`).
- malformed claim or record sender DID is rejected (`Regression: #453`).
- empty claim or record signatures are rejected (`Regression: #553`).

## Local Validation
Run from repository root:

```bash
cargo test -p kamn-core --test instruction_verification
cargo fmt --check
cargo clippy -- -D warnings
cargo test
```
