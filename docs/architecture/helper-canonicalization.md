# Helper Canonicalization Contracts

schema_version=kamn.docs.helper-canonicalization.v1
last_reviewed=2026-02-25
related_issue=#5935

## Scope

Issue `#5935` removes duplicated parser/encoding helper implementations across:

- `crates/kamn-kolme/src/*` JSON token parsing and URL percent-encoding paths.
- `crates/kamn-mcp-server/src/{protocol,dispatch}.rs` JSON escaping and field lookup paths.

## Canonical Modules

- `crates/kamn-kolme/src/json_scalar_policy.rs`
  - `parse_json_string_token` implements RFC 8259-compatible JSON string parsing.
  - `percent_encode_component` is the single percent-encoding helper for Kolme request paths.
- `crates/kamn-mcp-server/src/json_helpers.rs`
  - `escape_json` is the single JSON escaping helper for protocol/dispatch serialization.
  - `json_field_value` is the single nested field lookup helper for root/params/arguments.
  - `json_optional_string_field`, `json_required_string_field`, and `json_optional_u64_field` provide shared payload extraction.

## Regression Guards

- `crates/kamn-kolme/tests/duplicate_helper_inventory_contracts.rs`
  - Fails closed if duplicated local `parse_json_string` or `percent_encode` helpers reappear.
- `crates/kamn-mcp-server/tests/duplicate_helper_inventory_contract.rs`
  - Fails closed if duplicated local `escape_json` or `json_field_value` helpers reappear.

## Verification Evidence

- `cargo test -p kamn-kolme`
- `cargo test -p kamn-mcp-server`
- `cargo mutants --in-diff /tmp/issue5935.diff -p kamn-kolme -p kamn-mcp-server`
