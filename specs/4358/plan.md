# Plan: #4358 Rotation Preflight Taxonomy Contract

## Approach

1. Add RED assertions in deployment preflight policy checker tests for missing taxonomy markers and observed reason-value mapping.
2. Implement deterministic taxonomy constants + observed-value helper in checker output.
3. Keep current fail-closed checks unchanged; only add stable output contract fields.
4. Update `docs/security/key-management.md` with rotation preflight evidence matrix/taxonomy markers.
5. Validate with targeted scripts and repo quality gates.

## Affected Modules

- `scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py`
- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `docs/security/key-management.md`
- docs contract tests if required.

## Risks and Mitigations

- Risk: taxonomy value projection can drift from reason code list order.
  - Mitigation: explicit deterministic taxonomy reason ordering constants + subset projection helper.
- Risk: new assertions overconstrain unrelated failure paths.
  - Mitigation: assert presence of targeted reasons, not exact full reason list for NO-GO paths.
