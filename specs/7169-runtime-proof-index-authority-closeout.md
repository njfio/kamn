# 7169 Runtime Proof Index Authority Closeout

## Objective

Synchronize the canonical runtime-proof index with the four validation
artifacts merged by the settlement-authority closeout while keeping failed or
blocked external observations separate from proven KAMN runtime behavior.

## Inputs / Outputs

Inputs:
- the #7160 live chain-backed bridge finality runbook
- the #7163 bridge-authorized escrow settlement runbook
- the #7161 authoritative SDK, CLI, and MCP parity runbook
- the #7162 external A2A/x402 receipt-authority probe

Outputs:
- bounded entries for the three proven KAMN slices
- a separate external evidence section for the failed and blocked probe
- a hard-fail documentation contract covering links and claim boundaries

## Boundaries / Non-Goals

- Do not rerun or claim another funded transaction.
- Do not change runtime, adapter, protocol, or public API behavior.
- Do not present the external probe as KAMN service authority or a successful
  settlement.
- Do not claim generalized settlement, mainnet custody, consensus, or
  production readiness.
- Do not add dependencies or modify CI.

## Failure Modes

- The index omits a merged authority runbook.
- Bridge-authorized release is described as a second transfer.
- Adapter parity is presented as three independent funded live transfers.
- The external FAIL/BLOCKED probe appears inside the proven-runtime section.
- The external probe loses its no-approval or no-settlement boundary.

## Acceptance Criteria

- [x] The index links the #7160 live finality runbook with bounded wording.
- [x] The index links the #7163 bridge-authorized settlement runbook and states
      that release reuses the finalized bridge transfer.
- [x] The index links the #7161 authority-parity runbook and preserves its
      composed-proof limitation.
- [x] The index links the #7162 probe in a separate non-proof section with FAIL,
      BLOCKED, no-approval, and no-settlement boundaries.
- [x] A focused Rust docs contract fails when a link, classification boundary,
      or critical claim marker is removed.
- [x] Targeted tests, formatting, and lint/static checks pass.

## Files To Touch

- `specs/7169-runtime-proof-index-authority-closeout.md`
- `docs/validation/current-proven-runtime-slices.md`
- `crates/kamn-node/tests/runtime_proof_index_authority_closeout_contract.rs`

## Error Semantics

The documentation contract must fail with a marker-specific assertion when a
required runbook or bounded-claim phrase is absent. It must also fail if the
external probe appears before the end of the proven-runtime section.

## Test Plan

Red:
- Add a contract requiring all four links and their bounded markers.
- Assert that the external probe appears only after the proven-runtime section.
- Run the focused contract and record the expected failure.

Green:
- Add the minimum index entries and separate external evidence section.
- Run the focused contract until it passes.

Refactor:
- Keep marker lists and section-order checks small and self-documenting.
- Run formatting, Clippy, and the existing runtime-proof index contract.

Integration:
- Run both proof-index contracts against the real workspace documentation.
- Verify every indexed path exists on the final tree.

## Implementation Evidence

- The canonical index links all three bounded KAMN authority slices under
  `What Is Currently Proven`.
- The bridge-authorized entry states that release reuses the finalized bridge
  transfer and does not submit another settlement transaction.
- The adapter-parity entry identifies the composed live/deterministic proof and
  rejects a three-funded-transfer interpretation.
- The external probe is in a later non-proof section with `FAIL`, `BLOCKED`,
  no-approval, and no-settlement markers.
- The focused authority-closeout and existing runtime-index contracts pass
  against the same workspace documentation.
