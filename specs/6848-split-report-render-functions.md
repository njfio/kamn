# Objective
Split the oversized rendering functions in `crates/kamn-node/src/report_render.rs` into bounded helpers while preserving exact text and JSON report behavior for `NodeBootstrapReport` output.

## Inputs/Outputs
- Inputs:
  - `crates/kamn-node/src/report_render.rs`
  - existing `NodeBootstrapReport` structure and callers
  - current `render_bootstrap_report()` behavior for text and JSON modes
- Outputs:
  - bounded helper functions for text rendering and JSON rendering
  - regression tests and/or extraction contracts proving the functions stay below the 25-line cap without behavior drift

## Boundaries/Non-goals
- Do not change report schema or field semantics.
- Do not change `NodeBootstrapReport` structure.
- Do not change public CLI/API behavior beyond internal refactoring.
- Do not introduce new dependencies.

## Failure modes
- Text output drops or renames existing fields.
- JSON output changes key/value semantics unexpectedly.
- New helpers still exceed the 25-line function cap.
- Refactor introduces duplicated formatting logic or hidden fallback behavior.

## Acceptance criteria
- [ ] `render_text_report()` is reduced below the 25-line function limit.
- [ ] `render_json_report()` is reduced below the 25-line function limit.
- [ ] Output remains behaviorally equivalent for both text and JSON modes under regression tests.
- [ ] No extracted helper exceeds the 25-line function limit.
- [ ] `report_render.rs` remains within active file-size policy, or is further split if required.

## Files to touch
- `specs/6848-split-report-render-functions.md`
- `crates/kamn-node/src/report_render.rs`
- `crates/kamn-node/tests/report_render_extraction_contract.rs`
- optionally supporting test files if a dedicated regression harness is cleaner

## Error semantics
- Rendering remains deterministic and fail-closed.
- Missing optional report fields must continue to render using the existing `none` behavior rather than silently changing defaults.
- No new silent fallback behavior is allowed.

## Test plan
1. Add a red extraction contract that fails while `render_text_report()` and `render_json_report()` exceed the function-size limit.
2. Add regression coverage proving text and JSON rendering preserve current output markers for representative reports.
3. Refactor the renderers into bounded helpers.
4. Re-run the extraction contract and regression tests until green.
5. Re-run targeted `kamn-node` tests touching report rendering.
