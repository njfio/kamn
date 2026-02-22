# Spec: #5629 EVIDENCE Phase Activation

- Issue: #5629
- Milestone: R54 E2E Evidence Phase Activation
- Status: Reviewed
- Priority: P1

## Problem Statement
`phase_results` currently reports the `EVIDENCE` phase as a static `SKIP` placeholder, even when `evidence_contract` reports deterministic PASS/FAIL outcomes. This weakens phase-level coherence with the evidence contract.

## Scope
### In Scope
- Drive `EVIDENCE` phase step/status/details from `evidence_contract` pass/fail state.
- Keep run-output contract shape stable except semantic activation of existing phase fields.
- Preserve runtime marker contracts.

### Out of Scope
- Real evidence artifact generation.
- Teardown phase semantics (separate follow-up slice).
- External process orchestration changes.

## Acceptance Criteria
### AC-1 Evidence pass semantics
Given non-failing evidence path,
When run output is generated,
Then `EVIDENCE` phase status is `PASS` and evidence step detail reflects deterministic pass counts.

### AC-2 Evidence fail semantics
Given deterministic evidence failure path (`evidence-fail`),
When run output is generated,
Then `EVIDENCE` phase status is `FAIL` with fail-specific detail.

### AC-3 Lifecycle propagation
Given EVIDENCE phase pass/fail,
When lifecycle summary is computed,
Then phase totals and step totals reflect that status transition.

### AC-4 Contract stability
Given existing runtime marker contracts,
When evidence-phase activation is applied,
Then `runtime_*`, `spawn_*`, `process_*`, and live status contracts remain unchanged.

## Conformance Cases
- C-01 (AC-1): normal path emits `EVIDENCE` phase `PASS` with expected detail markers.
- C-02 (AC-2): evidence-fail path emits `EVIDENCE` phase `FAIL` with fail marker detail.
- C-03 (AC-3): lifecycle summary pass/fail counts reflect EVIDENCE phase activation.
- C-04 (AC-4): runtime marker contract tests remain green.

## Success Metrics
- `cargo test -p kamn-e2e-harness --test command_contract` green with new EVIDENCE-phase assertions.
- `cargo test -p kamn-e2e-harness` remains green.
