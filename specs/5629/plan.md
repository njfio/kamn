# Plan: #5629 EVIDENCE Phase Activation

## Approach
1. Extend phase-step builder to receive evidence contract status context.
2. Replace static EVIDENCE step (`SKIP`) with deterministic PASS/FAIL based on evidence contract outcome.
3. Ensure phase details string reflects evidence summary and fail marker path.
4. Add RED tests for EVIDENCE phase status/detail and lifecycle summary propagation.
5. Add docs artifact + docs contract test and update R54 milestone index references.

## Affected Modules
- `crates/kamn-e2e-harness/src/lib.rs`
- `crates/kamn-e2e-harness/tests/command_contract.rs`
- `docs/research/` (R54 artifact)
- `crates/kamn-e2e-harness/tests/` (new docs contract test)
- `specs/milestones/r54-e2e-evidence-phase-activation/index.md`

## Risks and Mitigations
- Risk: existing lifecycle total assertions may require updates.
  - Mitigation: RED tests first, then deterministic recalculation updates.
- Risk: semantic drift between evidence_contract and phase details.
  - Mitigation: derive both from same evidence status source.

## Interfaces / Contracts
- No new top-level run output fields.
- Changed semantics:
  - `phase_results[phase=EVIDENCE].status`
  - `phase_results[phase=EVIDENCE].steps[0].status/detail`

## ADR
- Not required.
