# Plan: Issue #5818 - Live S-05 Scenario Activation

- Issue: #5818
- Status: Completed
- Spec: `specs/5818/spec.md`

## Approach
1. Add RED conformance tests for driver-level `execute("S-05")` fail-closed behavior in sdk/cli/mcp modules.
2. Extend each driver's live-probe field/routing and environment-backed probe construction to include dedicated `S-05` execution helpers.
3. Implement `S-05` live helpers using existing escrow operations:
   - `fund_escrow` / `fund-escrow`
   - `release_escrow` / `release-escrow`
4. Update toggle-contract expectations where `S-05` transitions from non-live-bound to live-bound.
5. Run targeted tests, full harness regression, and format/lint gates.
6. Update milestone index and lifecycle markers for closure.

## Affected Artifacts
- `crates/kamn-e2e-harness/src/drivers/sdk_direct.rs`
- `crates/kamn-e2e-harness/src/drivers/cli_scripted.rs`
- `crates/kamn-e2e-harness/src/drivers/mcp_agent.rs`
- `crates/kamn-e2e-harness/tests/sdk_direct_live_toggle_contract.rs`
- `specs/5818/spec.md`
- `specs/5818/plan.md`
- `specs/5818/tasks.md`
- `specs/milestones/r52-e2e-live-runtime-integration-hardening/index.md`

## Risks and Mitigations
- Risk: S-05 helper assumptions diverge from deterministic local service semantics.
  - Mitigation: validate against current `fund_escrow` + `release_escrow` contracts.
- Risk: live-bound mapping change breaks existing non-live toggle expectations.
  - Mitigation: update toggle tests to use still-non-live scenarios and keep regression suite green.
- Risk: adding `specs/5818` breaches spec-volume cap guardrail.
  - Mitigation: preserve cap via bounded archived-pointer cleanup if `review_r53_docs_contract` fails.

## Verification Strategy
- RED: new S-05 conformance tests fail before routing/implementation.
- GREEN: targeted S-05 conformance tests pass after implementation.
- Regression: full `kamn-e2e-harness` tests, fmt, clippy, and `review_r53_docs_contract` guardrail.
