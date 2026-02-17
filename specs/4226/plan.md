# Plan — #4226

Status: Implemented

## Approach
- Implement checker constants + reason mapping helpers.
- Wire checker to lane wrapper output and JSON report augmentation.
- Validate with red tests from #4225, then docs/CI contract tests.

## Risks / Mitigations
- Risk: marker duplication/drift between checker and lane.
  - Mitigation: reuse checker output as lane augmentation source.
