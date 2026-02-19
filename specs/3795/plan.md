# Issue #3795 Plan

- Issue: #3795
- Status: In Progress
- Spec: `specs/3795/spec.md`

## Implementation Approach
1. Tighten live transport CI exclusion script assertions for selector-gated local-heavy workflow and `ci-tools` fast-mode leakage.
2. Synchronize transport-fault-matrix reason/marker docs text in `docs/ci/strategy.md` with current checker surface.
3. Add/extend Rust docs contract coverage for the strategy markers and reason taxonomy declarations.

## Affected Modules
- `scripts/ci/test_live_transport_fault_matrix_ci_exclusion_policy.sh`
- `docs/ci/strategy.md`
- `crates/kamn-core/tests/ci_strategy_docs.rs`

## Risks and Mitigations
- Risk: docs/tests diverge from runtime policy reason-code surface.
  - Mitigation: assert deterministic marker/reason snippets in docs contract tests and run exclusion test in the same lane.
- Risk: shell-surface regression.
  - Mitigation: keep shell edits minimal and run shell LOC/ratio/ratchet guardrails.

## Contracts and Interfaces
- CI exclusion contract retains:
  - local-heavy selector gate condition in `.github/workflows/ci-fast-gate.yml`
  - no live transport run-mode command in `scripts/ci/test_ci_tools.sh` fast-mode block
- Strategy docs contract retains:
  - live transport fault matrix run-mode exclusion statement
  - retry/reconnect marker parity references
  - reason-code taxonomy entries aligned with checker surface

## Verification Strategy
- RED: introduce selector/leakage/docs parity assertions that fail against current drift.
- GREEN: update script/docs/tests until RED assertions pass.
- VERIFY: run targeted exclusion + docs tests and shell guardrails.
