# Plan: Issue #4472

Status: Completed
Issue: #4472

## Approach

1. Add boundary constants and marker surface in go/no-go contract lane script.
2. Add explicit deep-lane local-heavy opt-in and runtime budget checks.
3. Align lane tests with deterministic boundary failure markers.
4. Add CI strategy docs incident boundary matrix and docs tests.

## Affected Modules

- `scripts/deploy/gonogo_evidence_contract_lane_contract.sh`
- `scripts/deploy/run_gonogo_evidence_deep_lane.sh`
- `scripts/deploy/test_run_gonogo_evidence_contract_lane.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks / Mitigations

- Risk: run-mode scripts regress due to new opt-in guard.
  - Mitigation: reuse existing `KAMN_GONOGO_GATE_LOCAL_OPT_IN` guard already required by
    go-no-go runtime lane run mode.
- Risk: mismatch between emitted markers and docs text.
  - Mitigation: enforce docs contract tests with exact marker assertions.
