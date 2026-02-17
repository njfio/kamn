# Plan: #4365 Taxonomy Output Implementation

## Approach

1. Define deterministic rotation preflight taxonomy constants in checker.
2. Add helper to project observed taxonomy value from current reason list.
3. Add output fields:
   - `rotation_preflight_reason_taxonomy_version`
   - `rotation_preflight_reason_codes_csv`
   - `rotation_preflight_reason_codes_value`
4. Emit markers in stdout for compatibility with lane/log parsing.
5. Update key-management docs with rotation preflight evidence matrix markers.
