# Issue #5285 Plan

## Objective
Kick off Phase-6 by adding deterministic retention-to-archival gate execution contracts that bridge M8 lifecycle preconditions with M10 archival eligibility decisions.

## Affected Modules
- `crates/kamn-core/src/data_layer_m8_compliance.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival_tests.rs` (or equivalent test module)
- `docs/plans/2026-02-19-data-layer-infrastructure-activation-plan.md`
- `docs/review/data-layer-roadmap.md`

## Approach
1. Add failing conformance tests first for eligible, legal-hold denied, and precondition-denied archival projections.
2. Implement minimal projection glue that composes M8 lifecycle state into M10 archival gate outcomes.
3. Ensure denied paths expose deterministic reason markers and stable policy mapping.
4. Run targeted verification (`fmt`, strict `clippy`, targeted tests), then open PR with AC mapping.

## Risks and Mitigations
- Risk: semantic drift between M8 lifecycle vocabulary and M10 archival gate states.
  - Mitigation: use explicit mapping helpers with reason-coded denial variants.
- Risk: over-accepting archival candidates when lifecycle preconditions are incomplete.
  - Mitigation: fail-closed defaults and regression tests around denied preconditions.
- Risk: tracker drift after Phase-5 closure.
  - Mitigation: update milestone/plan/roadmap artifacts in the same PR.

## Interfaces and Contracts
- Preserve existing M8 and M10 public type contracts; add explicit adapter mapping without breaking existing call sites.
- Keep reason markers deterministic and stable for governance policy checks.
- No shell-surface additions.

## ADR
- No ADR required for this scoped bridge; no new dependencies or protocol-level architectural changes.
