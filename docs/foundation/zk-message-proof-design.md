# ZK Message Proof Design Spike (Issue #62)

This spike evaluates feasible zero-knowledge (ZK) message-proof designs for PRD Phase 4 without shipping a production prover/verifier stack yet.

## Scope Delivered
- Deterministic architecture scoring for candidate proof systems.
- Explicit complexity and trust-assumption comparison.
- A phased implementation proposal aligned to Kolme execution constraints.
- Envelope witness-construction strategy for selective disclosure.

## Kolme Constraints That Shape Design
- Kolme has a **single active processor** producing blocks at a time, so the first verification insertion point is processor transaction validation.
- Non-processor nodes perform **deterministic re-execution**; witness generation and verifier results must be reproducible across nodes.
- Kolme blocks contain one transaction, so verifier cost per message must be bounded to protect latency/throughput targets.
- Watchdog behavior can validate execution fairness, making it a practical extension point for proof integrity checks.

## Architecture Options
| Option ID | Proof System | Topology | Verifier Latency | Proof Size | Delivery Estimate | Notes |
|---|---|---|---:|---:|---:|---|
| `groth16-processor-only` | Groth16 | Processor-only | 4ms | 192 bytes | 7 weeks | Fast verifier; requires trusted setup ceremony. |
| `plonkish-batched-envelope` | Plonkish | Validator quorum | 15ms | 896 bytes | 10 weeks | Transparent setup, batching-friendly, strong Phase 4 fit. |
| `stark-recursive-watchdog` | STARK | Watchdog sampling | 45ms | 4608 bytes | 14 weeks | Highest transparency; currently too heavy for Phase 4 budgets. |

## Complexity and Trust Assumptions
- **Groth16 path**:
  - Lowest verifier latency and proof size.
  - Depends on a trusted setup ceremony and ceremony transcript integrity.
- **Plonkish path**:
  - No trusted setup ceremony, moderate proving/verifying costs.
  - Better fit for batched envelope proofs in validator-quorum mode.
- **STARK path**:
  - Strong transparency model and recursion roadmap.
  - Current verifier and proof-size overhead shifts this toward post-Phase-4 hardening with watchdog sampling first.

## Recommended Phase 4 Rollout
- **Phase 4.0 - Feasibility harness**
  - Implement canonical-envelope witness derivation and deterministic commitment checks.
  - Validate policy boundaries and deterministic failure modes.
- **Phase 4.1 - Processor verification pilot**
  - Add verifier hook at processor transaction-validation stage.
  - Reject unverifiable proofs deterministically and emit explicit operator diagnostics.
- **Phase 4.2 - Validator and watchdog expansion**
  - Extend proof checks to validator quorum paths and watchdog sampling.
  - Add anomaly alerts for proof mismatches, replay attempts, and censorship-aligned drop patterns.

## Deterministic Evaluation Rules
- Option scoring uses fixed budgets:
  - verifier latency budget
  - proof-size budget
  - engineering-week budget
  - transparent-setup policy switch
- Feasibility requires deterministic witness inputs and passing all budgets.
- Policy and option validation errors are explicit and typed.
- Regression guard: threshold checks are inclusive (`<=`) for verifier latency, proof size, and delivery weeks.

## Validation and Error Handling
- Invalid evaluation policy (for example zero limits) is rejected before scoring.
- Invalid option payloads (for example zero proof size) are rejected with option-specific errors.
- Witness generation rejects:
  - invalid canonical envelopes
  - missing private fields
  - empty private-field selectors

## Processor Admission Guard Contract
- Processor admission consumes a deterministic proof artifact containing:
  - `artifact_id`
  - `message_id`
  - `payload_commitment`
  - `proof_value`
- Admission rejects and blocks state mutation when:
  - artifact message ID mismatches the targeted message
  - payload commitment mismatches the expected witness commitment (tampered artifact)
  - artifact ID is replayed
  - proof value fails deterministic verification format checks
- Regression guard: tampered processor proof artifacts are rejected before admission (`Regression: #509`).

## Fast and Cost-Effective Validation
Run the smallest lane needed for rapid feedback:

```bash
cargo test -p kamn-core --test zk_message_proofs --test zk_message_proofs_docs
cargo fmt --check
cargo clippy -- -D warnings
```

Then run the crate regression lane:

```bash
cargo test -p kamn-core
```
