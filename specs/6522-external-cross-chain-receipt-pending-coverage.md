## Objective

Add explicit `kamn-core` external regression coverage proving that direct
`CrossChainReceiptStatus::Pending` input normalizes to
`CrossChainReceiptFinality::Pending` through the public re-export surface.

## Inputs/Outputs

- Input: a `CrossChainReceiptProof` constructed in `kamn-core` tests with
  `status = CrossChainReceiptStatus::Pending`
- Output: `normalize_cross_chain_receipt(&proof)` returns `Ok(...)` with
  `finality = CrossChainReceiptFinality::Pending`

## Boundaries/Non-goals

- No changes to `kamn-bridges` normalization logic
- No changes to public types or receipt normalization semantics
- No new workflow, CI, or dependency changes
- No broad cross-chain settlement expansion beyond this public-surface
  regression

## Failure modes

- The public `kamn-core` re-export path no longer normalizes direct
  `Pending` status to `CrossChainReceiptFinality::Pending`
- The external regression is missing, allowing the public surface to drift from
  the internal `kamn-bridges` unit coverage
- The added test creates file-size or formatting regressions

## Acceptance criteria

- [ ] `kamn-core` has a targeted external regression asserting direct
      `CrossChainReceiptStatus::Pending` input normalizes to
      `CrossChainReceiptFinality::Pending`
- [ ] The red phase demonstrates the external regression was absent before the
      new test was added
- [ ] `cargo test -p kamn-core --test cross_chain_receipt_finality -- --nocapture`
      passes locally
- [ ] This spec records that the issue is coverage hardening only, with no
      intended production behavior change

## Files to touch

- `specs/6522-external-cross-chain-receipt-pending-coverage.md`
- `crates/kamn-core/tests/cross_chain_receipt_finality.rs`

## Error semantics

- No new runtime errors are introduced
- If normalization unexpectedly regresses, the external test must fail with a
  direct assertion mismatch rather than silently tolerating behavior drift

## Test plan

1. Add a red-phase contract check in `crates/kamn-core/tests/cross_chain_receipt_finality.rs`
   that proves the external pending-normalization regression is not yet pinned.
2. Add the targeted external regression test for direct `Pending` status
   normalization through `kamn-core`.
3. Run `cargo test -p kamn-core --test cross_chain_receipt_finality -- --nocapture`.
