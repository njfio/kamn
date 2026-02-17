# Plan: #4170 Deterministic Custody Reason Mapping Contract

## Approach

1. Introduce custody taxonomy constants and deterministic observed-value projection helper in the deployment preflight checker.
2. Emit custody taxonomy markers in both JSON output and stdout marker stream.
3. Add focused test assertions for custody taxonomy fields and deterministic mismatch mapping order.
4. Add release checklist gate section for custody reason mapping and attach docs-contract tests.

## Affected Modules

- `scripts/kolme/check_local_kolme_live_deployment_preflight_policy.py`
- `scripts/kolme/test_check_local_kolme_live_deployment_preflight_policy.sh`
- `docs/foundation/release-gonogo-checklist.md`
- `crates/kamn-core/tests/release_gonogo_checklist_docs.rs`

## Risks and Mitigations

- Risk: marker additions could unintentionally break existing consumers.
  - Mitigation: keep existing rotation taxonomy fields unchanged; add new custody markers as additive output.
- Risk: reason-code ordering drift.
  - Mitigation: fixed constant ordering and deterministic projection helper.
