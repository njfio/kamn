# Plan — #4241 Deterministic Append-Checkpoint Outputs

Status: Reviewed

## Approach

1. Add append/checkpoint integrity marker constants and output fields in checker.
2. Add policy checks for marker integrity and append-checkpoint parity mismatch.
3. Project marker fields into policy JSON/stdout and contract-lane passthrough tests.
4. Update docs and docs tests for release/ops marker parity.

## Risks

- Breaking existing tests if markers are renamed.
  Mitigation: additive marker introduction and aligned test updates in same change.
