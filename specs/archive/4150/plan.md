# Issue #4150 Plan

- Issue: #4150
- Status: Implemented

## Approach
1. Add RED docs-contract assertions in `release_gonogo_checklist_docs.rs` for:
   - deployment preflight marker contract section presence,
   - required marker taxonomy/version/reason-codes fields,
   - fail-closed schema drift reason markers.
2. Update `docs/foundation/release-gonogo-checklist.md` with deployment preflight marker completeness contract text aligned to the new assertions.
3. Run scoped docs-contract suite and keep diffs limited to Rust tests + docs/spec artifacts only.

## Affected Modules
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`
- `docs/foundation/release-gonogo-checklist.md`
- `specs/4150/spec.md`
- `specs/4150/plan.md`
- `specs/4150/tasks.md`

## Risks and Mitigations
- Risk level: high
- Mitigations:
  - Use exact stable marker strings to keep drift detection deterministic.
  - Keep section naming and regression anchors explicit to avoid accidental contract loss.
  - Restrict scope to docs-contract validation (no runtime behavior changes).

## Interface Contract
- No API changes.
- No dependency or protocol changes.

## ADR
- Not required for this scoped docs-contract subtask.

## Verification Summary
- RED baseline: the new `checklist_contains_deployment_preflight_marker_completeness_schema_drift_gate` assertions fail against a checklist variant that omits the new deployment-preflight marker section/strings.
- GREEN: `cargo test -p kamn-core --test release_gonogo_checklist_docs` (94 passed, 0 failed).
- Regression: `cargo fmt --check` (pass).
