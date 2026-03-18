# 7009 External-Chain-Backed Settlement Lane

## Objective
Implement one honest bounded external-chain-backed settlement lane on current `main`. The lane must make external chain evidence drive escrow settlement semantics on the public KAMN runtime surface, instead of proving local-only escrow release beside an unrelated live bridge evidence path. The proof must stay explicit about what it does not prove.

## Inputs/Outputs
- Inputs:
  - one checked-in live Solana-backed evidence source already available on current `main`
  - one escrow release path on the public service-api or harness-backed runtime surface
  - one persisted settlement evidence projection that survives restart
- Outputs:
  - one runtime path where release or refund state derives from external-chain-backed evidence rather than local-only release mutation
  - one operator-facing validation runbook under `docs/validation/`
  - one hard-fail docs contract guarding the runbook and proof-index entry
  - at least one integration or e2e proof exercising the real path end-to-end

## Boundaries/Non-goals
- Do not claim broad production readiness.
- Do not claim Byzantine-safe dispute resolution.
- Do not claim bridge finality beyond the bounded finality proofs already on `main`.
- Do not publish another evidence-only wrapper that leaves escrow settlement semantics local-only.
- Do not widen this issue into a general bridge or escrow rewrite.

## Failure Modes
- external chain evidence is gathered but never consumed by escrow settlement state
- the release path still returns a local-only synthetic state with no persisted external evidence linkage
- settlement evidence persists placeholder or synthetic receipt material instead of external-chain-backed evidence
- restart loses the external settlement evidence linkage
- the runbook overstates the claim as external economic settlement or finality when the implementation only proves evidence-coupled settlement semantics

## Acceptance Criteria (testable booleans)
- [ ] one real release or refund path consumes external-chain-backed evidence before emitting the terminal escrow state
- [ ] the resulting escrow settlement state persists a non-placeholder external evidence linkage that remains queryable across restart
- [ ] one integration or e2e proof exercises the real settlement lane end-to-end and fails if the external evidence linkage is missing or placeholder
- [ ] one validation runbook exists at `docs/validation/` with explicit `What This Proves` and `What This Does Not Prove` sections
- [ ] one hard-fail docs contract fails when the runbook or proof-index marker is missing or overstates the claim
- [ ] `docs/validation/current-proven-runtime-slices.md` links the new runbook

## Files to Touch
- `specs/7009-external-chain-backed-settlement-lane.md`
- `docs/validation/*external-chain*settlement*.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/*settlement*slice*contract*.rs`
- `crates/kamn-node/src/service_api_endpoint/message_store/store/task_escrow_ops.rs`
- one bounded settlement-evidence helper under `crates/kamn-node/src/service_api_endpoint/` or `crates/kamn-core/src/data_layer_m4_escrow_integration/`
- one integration or e2e proof under `crates/kamn-node/src/main_tests/` or `crates/kamn-e2e-harness/tests/`

## Error Semantics
- startup must fail loud if the external-chain-backed settlement lane is enabled with invalid or empty live config
- the settlement path must fail hard when external evidence collection fails, schema-check fails, or required finalized evidence is absent
- no silent fallback to the existing local-only release path is allowed when the external-chain-backed lane is explicitly selected
- boundary handlers may translate typed failures into response envelopes, but interior code must return structured errors and preserve cause

## Test Plan
1. Red:
   - add docs contract for the missing runbook and proof-index markers
   - add one integration or e2e proof that expects non-placeholder external settlement evidence on the release path
   - confirm both fail on current `main`
2. Green:
   - implement the smallest honest settlement-evidence linkage from the existing live Solana proof lane into escrow release or refund semantics
   - persist and expose the resulting evidence linkage
   - publish the runbook and proof-index entry
3. Refactor:
   - split helpers at logical seams so no touched file exceeds repo limits
   - remove duplication between the new settlement lane and the existing live bridge evidence collector where possible
4. Verification:
   - docs contract
   - real integration or e2e proof
   - any touched split-contract coverage
   - touched-Rust policy gate

## Notes
- Local inspection on 2026-03-18 showed that current `main` has two separate bounded capabilities:
  - local-heavy live S-05 escrow settlement
  - live Solana-backed bridge evidence collection
- This issue exists to connect those capabilities honestly, or fail loudly if that connection cannot be implemented cleanly.
