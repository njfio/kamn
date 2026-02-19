# Issue #5229 Plan

- Issue: #5229
- Milestone: specs/milestones/r27-47-r43-gap-remediation-and-delivery-rebalancing/index.md

## Approach
1. Introduce typed DID boundary wrappers across wave-B modules while preserving existing public string-based data contracts where required.
2. Normalize DID validation errors to structured deterministic markers (`field`, `reason_code`, `detail`) for all wave-B surfaces.
3. Replace stringly propagation from operator binding into operator actions with typed error passthrough.
4. Add deterministic invalid-DID regression coverage for operator binding/actions, dashboard API/UI, governance workflow, and task payment workflow.
5. Run targeted wave-B suites, formatting/lint gates, and shell-ratio guardrail check.

## Affected Modules
- `operator_binding.rs`: validated agent/operator DID wrappers and deterministic invalid-DID errors.
- `operator_actions.rs`: preserve API shape but use typed binding errors and deterministic invalid-DID outcomes.
- `operator_dashboard_api.rs`: validated DID boundaries for agent/task/message/escrow/reputation upserts and structured invalid-DID taxonomy.
- `operator_dashboard_ui.rs`: add DID boundary checks for rendered snapshot/audit records and deterministic invalid-DID taxonomy.
- `governance_workflow.rs`: typed proposer/voter/executor DID boundary conversions and structured invalid-DID errors.
- `task_payment.rs`: typed payer/payee/confirmer DID boundary conversions and structured invalid-DID errors.

## Risks and Mitigations
- Risk: Broad error-enum updates break existing tests and call sites.
  - Mitigation: preserve non-DID variants and update tests in one commit with clear reason-code assertions.
- Risk: UI/API behavior drift from new validation points.
  - Mitigation: keep validation scoped to existing DID fields and maintain existing success-path behavior.
- Risk: shell ratio regression.
  - Mitigation: rust-only implementation and run `scripts/ci/test_check_shell_rust_ratio_guardrail.sh`.

## Interfaces / Contracts
- Invalid DID variants in wave-B modules expose:
  - `field`: failing input field name
  - `reason_code`: deterministic parser marker
  - `detail`: parser detail text
- Typed conversion boundaries use `TryFrom<&RawType>` wrappers where input structs must remain string-based.
- Operator actions retain external signatures and map authorization failures through typed `OperatorBindingError` instead of opaque string payloads.
