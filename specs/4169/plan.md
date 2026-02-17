# Plan: #4169 Quorum Marker Parity and Tamper Rejection Tests

## Approach

1. Add targeted RED-style fixture mutations in deployment preflight checker tests for:
   - quorum approval-count parity drift,
   - quorum/custody digest tamper mapping.
2. Assert deterministic fail-closed reasons for those paths.
3. Update `docs/deploy/kolme_devnet_ops.md` compatibility runbook section with marker taxonomy + commands.
4. Add docs-contract tests for deploy compatibility markers.

## Affected Modules

- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `docs/deploy/kolme_devnet_ops.md`
- `crates/kamn-core/tests/kolme_devnet_ops_docs.rs`

## Risks and Mitigations

- Risk: New assertions overconstrain unrelated failure paths.
  - Mitigation: check for required marker presence, not full unordered failure payload equality.
- Risk: docs compatibility file drifts from planning/source runbook.
  - Mitigation: add explicit docs-contract tests for new section markers and regression references.
