# Spec: Issue #5986 - Replace M10 archival checksum placeholder with real SHA-256

- Issue: #5986
- Status: Accepted (agent-authored P2 self-acceptance)
- Type: task
- Priority: P2
- Area: backend
- Milestone: `specs/milestones/r66-r57-residual-gap-closure/index.md`
- Last Updated: 2026-02-25

## Problem Statement
`deterministic_checksum_marker` in `crates/kamn-core/src/data_layer_m10_partition_archival/shared.rs` currently emits a formatted placeholder (`sha256:<partition_name>:<partition_month_id>`) rather than hashing payload bytes. This mislabels checksum integrity output and weakens archival index trust semantics.

## Scope
In scope:
- Compute actual SHA-256 digest for M10 partition archival checksum markers.
- Preserve deterministic output format `sha256:<hex_digest>`.
- Update unit coverage to assert digest correctness against known vectors and non-placeholder behavior.

Out of scope:
- M10 schema redesign.
- Storage backend changes.

## Risk Level
`med`

## Acceptance Criteria
- AC-1: `deterministic_checksum_marker` returns a real SHA-256 digest in `sha256:<hex>` format.
- AC-2: The checksum marker remains deterministic for identical inputs and changes when inputs change.
- AC-3: Existing M10 archival flows continue compiling and passing with updated checksum semantics.
- AC-4: Regression test prevents reintroduction of placeholder string interpolation output.

## Conformance Cases
- C-01 (Unit, AC-1): Known input vector maps to expected SHA-256 digest marker.
- C-02 (Unit, AC-2): Same inputs produce identical marker; changed month/name produces different marker.
- C-03 (Functional, AC-3): M10 archival registry tests pass without behavior regressions.
- C-04 (Regression, AC-4): Placeholder-shaped `sha256:<name>:<month>` output is explicitly rejected.

## Success Metrics / Observable Signals
- Targeted M10 module tests pass.
- `cargo test -p kamn-core data_layer_m10_partition_archival` remains green.
- No formatting/clippy regressions in touched files.
