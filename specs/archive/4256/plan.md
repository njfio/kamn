# Plan — #4256 Partition-Healing Mismatch Reason Mapping

Status: Implemented

## Approach

1. Define deterministic mismatch mapping taxonomy/version/codes constants.
2. Add resolver from `failed_checks` to one stable mismatch reason category.
3. Emit mapping markers in policy JSON + CLI output.
4. Wire mapping marker assertions into policy/lane tests and docs markers.

## Risks

- Existing consumers may parse strict output lines.
  - Mitigation: append new output markers while preserving existing lines.
