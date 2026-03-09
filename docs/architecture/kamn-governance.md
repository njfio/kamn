# kamn-governance Architecture

## Intent
Defines the first extracted governance boundary pulled out of `kamn-core` during
Phase 1 decomposition work.

## Identity Markers
- `kamn_governance_phase1_scope=governance_workflow,operator_binding,operator_actions`
- `kamn_governance_phase1_retained_in_core=operator_dashboard_api,operator_dashboard_ui`
- `kamn_governance_dependency_policy=no-kamn-core`

## Responsibilities
- Own governance proposal, vote, evaluation, and execution workflow contracts.
- Own operator binding authorization and proof validation.
- Own permissioned operator action auditing and mutation controls.

## Boundaries
- Depends on `kamn-types` for foundational DID parsing/types.
- Must not depend on `kamn-core`.
- Leaves dashboard projection modules in `kamn-core` until task, escrow,
  message, and reputation domain projections are extracted separately.

## Operational Notes
- Stable `kamn-core` public paths remain available through compatibility shims in:
  - `crates/kamn-core/src/governance_workflow.rs`
  - `crates/kamn-core/src/operator_binding.rs`
  - `crates/kamn-core/src/operator_actions.rs`
- Primary validation path:
  - `cargo test -p kamn-core --test governance_workflow`
  - `cargo test -p kamn-core --test operator_permissioned_actions`

## Related
- `docs/architecture/kamn-core-target-crate-graph.md`
- `docs/architecture/kamn-core-module-map.md`
