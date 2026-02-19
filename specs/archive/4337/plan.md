# Plan — #4337

Status: Reviewed

## Approach

- Introduce module-boundary checker helpers in `local_full_stack_integration_live_contract.py` that inspect source boundaries quickly.
- Add checker outputs into run-lane payload and policy validation.
- Extend reason normalization with module-boundary reason fields while preserving existing runtime phase parity outputs.
- Update docs and docs-contract tests.

## Risks and Mitigations

- Risk: checker becomes brittle against harmless refactors.
  - Mitigation: assert boundary ownership markers, not incidental formatting.
- Risk: regression to existing runtime phase parity contract behavior.
  - Mitigation: keep existing phase taxonomy fields intact and additive-check module-boundary fields.

## Interfaces and Contracts

- `runtime_module_boundary_parity_reason_taxonomy_version=kamn.runtime.module-boundary-parity-reason-taxonomy.v1`
- `runtime_module_boundary_parity_reason_codes_csv=<deterministic csv>`
- `runtime_module_boundary_reason_codes_value=none|<csv>`
