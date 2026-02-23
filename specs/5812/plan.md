# Plan: Issue #5812 - Live S-02 Matrix Execution Evidence

- Issue: #5812
- Status: Completed
- Spec: `specs/5812/spec.md`

## Approach
1. Add RED docs-contract assertions in the existing release docs-contract suite for a new live S-02 evidence artifact and milestone linkage.
2. Create/update evidence artifact with deterministic marker schema and required fields for per-mode and per-scenario status.
3. Execute live harness matrix in one persistent shell lifecycle with local `kamn-node` runtime, covering `sdk-direct`, `cli-scripted`, and `mcp-tau` on `S-01,S-02,S-04,S-06`.
4. Populate observed results/markers from command outputs and satisfy docs-contract assertions.
5. Preserve spec-volume non-regression cap if adding `specs/5812` triggers `review_r53_docs_contract` failure.
6. Update milestone index slice markers and complete lifecycle closeout.

## Affected Artifacts
- `crates/kamn-e2e-harness/tests/docs_contract_release_group.rs`
- `docs/research/e2e-live-testing-prd-r55-live-s02-execution-evidence.md`
- `specs/5812/spec.md`
- `specs/5812/plan.md`
- `specs/5812/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`
- `specs/3910/ARCHIVED.md` (legacy archived pointer removal for cap preservation)

## Risks and Mitigations
- Risk: live runtime/process lifecycle instability causes false negative harness outcomes.
  - Mitigation: run node + three harness commands in one persistent shell session and capture structured outputs under `.tmp/5812-live/`.
- Risk: S-02 live path regression appears during true runs.
  - Mitigation: treat as fail-closed and patch driver/service contracts in-slice if needed before closure.
- Risk: spec-volume cap regression (top-level `specs/` dir count) after adding `specs/5812`.
  - Mitigation: preserve cap via bounded archived-pointer cleanup if required by docs-contract guardrail.

## Verification Strategy
- RED: `cargo test -p kamn-e2e-harness --test docs_contract_release_group -- --nocapture` fails before marker/doc updates.
- GREEN: same lane passes after updates.
- Integration: live matrix execution commands for all three modes with `S-02` included.
- Regression: `cargo test -p kamn-e2e-harness -- --nocapture`, `cargo fmt --check`, and scoped clippy.
