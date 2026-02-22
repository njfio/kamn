# Spec: #5693 Mutation Hardening for `kamn-mcp-server` Protocol Helpers

- Issue: #5693
- Milestone: r52-e2e-live-runtime-integration-hardening
- Status: Implemented
- Priority: P2

## Problem Statement
Issue #5692 introduced framed MCP stdio protocol handling and passed functional
tests, but mutation testing reported `61/89` mutants caught (`28` missed) for
the in-diff slice. The missed set clusters around helper logic in
`crates/kamn-mcp-server/src/protocol.rs` (ID normalization, frame parsing edge
conditions, numeric extraction, and JSON escaping branches).

## Scope
### In Scope
- Add targeted tests for helper branches currently weakly covered in
  `protocol.rs`.
- Improve mutation resistance for the #5692 diff region.
- Keep protocol behavior and public API unchanged.

### Out of Scope
- New tool semantics or JSON-RPC method additions.
- Dependency changes.
- Large parser redesign.

## Acceptance Criteria
### AC-1 Branch coverage hardening
Given escaped helper-level mutants from #5692,
When targeted tests are added,
Then branches for ID normalization, frame parsing, numeric parsing, and escaping
are directly asserted.

### AC-2 Mutation outcome improvement
Given baseline mutation result `61/89` caught,
When mutation gate is rerun for the in-diff scope,
Then caught mutants increase and missed mutants decrease versus baseline.

### AC-3 Regression safety
Given existing `kamn-mcp-server` contract suites,
When hardening changes are applied,
Then all crate tests continue to pass.

## Conformance Cases
- C-01 (AC-1): unit tests assert dispatch-id normalization behavior across quoted,
  numeric, null, and empty IDs.
- C-02 (AC-1): unit tests assert framed decoder rejects missing/invalid
  content-length and accepts valid single/multi-frame streams.
- C-03 (AC-1): unit tests assert u64 field extraction for numeric and quoted
  numeric forms and rejection of invalid forms.
- C-04 (AC-1): unit tests assert JSON escaping paths for quote, slash, newline,
  carriage-return, and tab.
- C-05 (AC-2): mutation run shows improved caught/missed ratio versus `61/89`.
- C-06 (AC-3): `cargo test -p kamn-mcp-server` remains green.

## Success Metrics / Observable Signals
- Mutation delta improved from baseline (`61 caught / 28 missed`).
- `cargo test -p kamn-mcp-server` passes.
- `cargo fmt --all --check` and `cargo clippy -p kamn-mcp-server -- -D warnings`
  pass.
