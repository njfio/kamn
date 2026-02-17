# Issue #4380 Plan

Status: Reviewed

## Approach

1. Add native signer taxonomy constants and evaluation logic in `check_local_signed_to_kolme_demo_policy.py`.
2. Return native signer taxonomy value from evaluation and include it in JSON/stdout report output.
3. Add deterministic runtime signing profile fields to signed-to-Kolme summary output in `local_signed_to_kolme_demo_contract_lane.py`.
4. Update docs and docs contract assertions for new taxonomy output markers.

## Risks

- Contract drift with existing runtime checker expectations.
  - Mitigation: reuse existing canonical profile marker/value (`kolme-fork-secp256k1-v1`) already enforced in runtime integration policy.
