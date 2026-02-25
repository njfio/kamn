# Plan: Issue #5986

## Approach
- Replace `deterministic_checksum_marker` implementation with `tagged_sha256` over canonical input payload.
- Canonical payload: `<partition_name>:<partition_month_id>` (stable ordering, deterministic separator).
- Update/extend tests in `shared.rs` to assert:
  - exact expected digest marker for known vector,
  - deterministic same-input behavior,
  - rejection of placeholder output shape.
- Run scoped M10 tests and full targeted crate slice.

## Affected Modules
- `crates/kamn-core/src/data_layer_m10_partition_archival/shared.rs`
- `crates/kamn-core/src/data_layer_m10_partition_archival/registry.rs` (indirect verification via tests)

## Risks / Mitigations
- Risk: changing marker string could break tests expecting placeholder format.
  Mitigation: update tests to real digest vector and keep stable `sha256:` prefix.
- Risk: accidental digest input drift.
  Mitigation: pin exact known-vector assertion in regression test.

## Interfaces / Contracts
- Public contract retained: function returns `String` marker beginning with `sha256:`.
- Semantics contract changes from placeholder marker to cryptographic digest marker.
