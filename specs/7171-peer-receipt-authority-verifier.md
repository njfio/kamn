# 7171 Peer Receipt-Authority Verifier

## Objective

Add a reusable offline verifier that proves or rejects the complete peer
request, challenge, approval, settlement, and service-result receipt chain
without trusting ambient actor evidence or opaque peer assertions.

## Inputs / Outputs

Inputs:
- canonical request and result bytes
- request, challenge, approval, settlement, and result digest claims
- stable challenge ID, nonce, expiry, and stage timestamps
- payer, payee, asset, network, and amount claims at every economic stage
- authoritative settlement receipt ID and transaction ID
- the existing normalized #7162 external observation

Outputs:
- a typed `Pass`, `Fail`, or `Blocked` verdict
- stable machine-readable error code
- failing stage and field context
- recomputed domain-separated SHA-256 commitments

## Boundaries / Non-Goals

- This is an offline conformance verifier, not an x402 wire-format standard.
- A synthetic passing chain proves verifier behavior, not external
  interoperability or a live settlement.
- Do not spend funds, use credentials, or call the external peer again.
- Do not change KAMN service, SDK, CLI, MCP, bridge, or settlement behavior.
- Do not add dependencies or modify CI.
- Do not infer missing evidence from matching actor or payment terms.

## Canonical Digest Contract

- Request digest: SHA-256 over the exact canonical request bytes.
- Challenge digest: domain `kamn.peer.challenge.v1`, then length-prefixed request
  digest, challenge ID, nonce, expiry, payer, payee, asset, network, and amount.
- Approval digest: domain `kamn.peer.approval.v1`, then length-prefixed request
  and challenge digests, challenge ID, nonce, approval timestamp, and the same
  economic fields.
- Settlement digest: domain `kamn.peer.settlement.v1`, then length-prefixed
  request, challenge, and approval digests, receipt ID, transaction ID,
  finalized timestamp, and the same economic fields.
- Service-result digest: domain `kamn.peer.result.v1`, then length-prefixed
  request and settlement digests plus exact canonical result bytes.

All digest strings use lowercase `sha256:` plus 64 hexadecimal characters.

## Failure Modes

- A required stage or field is absent.
- A digest has malformed encoding or does not recompute.
- Request, challenge, approval, settlement, or result identities disagree.
- Challenge ID or nonce changes after the challenge.
- Approval occurs after absolute challenge expiry.
- Settlement predates approval or result predates settlement.
- Payer, payee, asset, network, or amount changes between stages.
- Settlement receipt ID or transaction ID is absent.
- The external #7162 observation is promoted to `Pass` despite missing stages.

## Acceptance Criteria

- [ ] `kamn-e2e-harness` exports the evidence model, digest builders, verifier,
      verdict, and structured error.
- [ ] A deterministic complete synthetic chain evaluates to `Pass`.
- [ ] Every digest is recomputed from the canonical fields defined above.
- [ ] Missing stages and fields fail with `PEER_AUTHORITY_STAGE_MISSING` or
      `PEER_AUTHORITY_FIELD_MISSING`.
- [ ] Malformed and recomputation-mismatched digests fail with
      `PEER_AUTHORITY_DIGEST_INVALID` or `PEER_AUTHORITY_DIGEST_MISMATCH`.
- [ ] Identity and economic mutations fail with
      `PEER_AUTHORITY_BINDING_MISMATCH`.
- [ ] Expiry and stage-order mutations fail with
      `PEER_AUTHORITY_TIME_INVALID`.
- [ ] The #7162 fixture is evaluated through the shared verifier and remains
      non-passing with settlement visibility `Blocked`.
- [ ] No fixture, test output, or model includes credentials or private keys.
- [ ] Focused tests, existing #7162 tests, formatting, and Clippy pass.

## Files To Touch

- `specs/7171-peer-receipt-authority-verifier.md`
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/src/peer_receipt_authority.rs`
- `crates/kamn-e2e-harness/src/peer_receipt_authority_digest.rs`
- `crates/kamn-e2e-harness/src/peer_receipt_authority_validate.rs`
- `crates/kamn-e2e-harness/tests/peer_receipt_authority_contract.rs`
- `crates/kamn-e2e-harness/tests/peer_receipt_authority_contract/support.rs`
- `crates/kamn-e2e-harness/tests/external_a2a_x402_receipt_authority_contract.rs`

## Error Semantics

`PeerReceiptAuthorityError` includes `code`, `message`, `stage`, `field`,
context, and an optional cause. Expected conformance failures use typed errors;
parsing failures preserve their source text as the cause. Interior validation
does not log. Missing or malformed evidence never falls back to `Pass`.

`Blocked` means the observation explicitly stopped before settlement because
of a no-funds or no-credentials boundary. It does not erase an earlier
challenge failure: the verdict retains the validation error and reports
settlement visibility separately.

## Test Plan

Red:
- Compile-time public API contract.
- Complete synthetic chain PASS contract.
- Missing-stage and missing-field table.
- Digest mutation table.
- Identity/economic mutation table.
- Expiry/order mutation table.
- Existing #7162 fixture integration contract.

Green:
- Add the minimum public evidence types and digest builders.
- Validate each stage sequentially and fail on the first unsupported claim.
- Map the normalized external fixture into the shared attempt model.

Refactor:
- Separate digest construction from binding validation.
- Centralize shared economic and digest checks.
- Keep every file under 200 lines and every function under 25 lines.

Integration:
- Run both the new verifier contract and the existing #7162 contract.
- Verify the real fixture remains non-passing and no external call occurs.
- Run package tests, formatting, Clippy, and secret-pattern checks.
