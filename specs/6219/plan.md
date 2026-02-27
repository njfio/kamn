# Issue 6219 Plan

## Approach
1. Add public helper functions to `crates/kamn-cli/src/lib.rs`:
   - `is_help_request<I, S>(args: I) -> bool`
   - `render_help_text() -> String`
2. Implement helpers by reusing existing constants and `help_output()` path to avoid duplicate logic.
3. Add focused unit tests for helper behavior.
4. Run targeted build/tests for regression verification.

## Affected Modules
- `crates/kamn-cli/src/lib.rs`

## Risks and Mitigations
- Risk: behavior drift between helper text and dispatch help output.
  - Mitigation: implement `render_help_text()` via `help_output().text`.

## Interfaces
- Public API surface restored at crate root for binary caller compatibility.
