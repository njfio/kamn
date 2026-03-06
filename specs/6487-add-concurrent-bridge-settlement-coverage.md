## Objective

Add explicit concurrent evaluation coverage for `kamn-bridges` settlement decisions so mixed
cross-chain receipt proofs are verified to remain deterministic and independent when evaluated in
parallel.

## Inputs/Outputs

- Inputs:
  - `evaluate_cross_chain_settlement_decision(proof)`
  - `CrossChainReceiptProof`
  - parallel execution via `std::thread`
- Outputs:
  - deterministic settlement decisions for mixed proof batches under concurrent evaluation
  - deterministic repeated `Settle` decisions for the same final proof under concurrent evaluation

## Boundaries/Non-goals

- No production API changes
- No workflow, CI, or dependency changes
- No new batch settlement API in this slice
- No performance benchmarking beyond correctness-oriented concurrent tests

## Failure modes

- Mixed proofs evaluated concurrently produce incorrect per-proof decisions
- Concurrent evaluation leaks one proof’s result into another proof’s decision path
- Invalid proofs evaluated concurrently lose their typed rejection reason
- Repeated concurrent evaluation of the same final proof yields inconsistent results

## Acceptance criteria

- [x] A test proves concurrent evaluation of mixed final, pending, failed, and invalid proofs yields the expected decision for each proof
- [x] A test proves concurrent evaluation of the same final proof across multiple threads yields identical `Settle` decisions
- [x] A test proves concurrent evaluation of mixed pending and invalid proofs preserves pending decisions and typed invalid-proof rejection reasons
- [x] `cargo test -p kamn-bridges -- --nocapture` passes

## Files to touch

- `specs/6487-add-concurrent-bridge-settlement-coverage.md`
- `crates/kamn-bridges/tests/concurrent_bridge_settlement_contract.rs`
- `crates/kamn-bridges/tests/concurrent_bridge_settlement_integration.rs`
- `fixtures/ci/test_file_size_policy_baseline.env` (only if new test targets change inventory)

## Error semantics

- `evaluate_cross_chain_settlement_decision(...)` continues to return
  `CrossChainSettlementDecision::Reject(CrossChainSettlementRejectionReason::InvalidProof(...))`
  for invalid proofs
- Pending proofs continue to map to `CrossChainSettlementDecision::DeferPendingFinality`
- No new error or decision variants are introduced

## Test plan

- Add a contract test that requires a dedicated concurrent settlement integration target
- Add integration tests that evaluate mixed proof sets in parallel and assert deterministic
  per-proof decisions and typed rejection reasons
- Run:
  - `cargo test -p kamn-bridges --test concurrent_bridge_settlement_contract -- --nocapture`
  - `cargo test -p kamn-bridges --test concurrent_bridge_settlement_integration -- --nocapture`
  - `cargo test -p kamn-bridges -- --nocapture`
  - `cargo test -p kamn-core --test test_file_size_policy -- --nocapture` if test inventory changes

## Phase 6 integration evidence

- The real crate path is exercised via `cargo test -p kamn-bridges -- --nocapture`
- New integration coverage landed in
  `crates/kamn-bridges/tests/concurrent_bridge_settlement_integration.rs`
- Contract coverage landed in
  `crates/kamn-bridges/tests/concurrent_bridge_settlement_contract.rs`
- The test inventory baseline was refreshed to account for the two new test targets

## Deviations

- None
