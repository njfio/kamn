# Treasury Disbursement Approval Contracts (Issue #716)

This document defines deterministic treasury disbursement approval evidence
required before executing treasury payouts.

## Scope Delivered

- Treasury disbursement evidence generator and policy checker wrappers:
  - `scripts/treasury/generate_treasury_disbursement_evidence_bundle.sh`
  - `scripts/treasury/check_treasury_disbursement_policy.sh`
- Shared Python implementation backing the wrappers:
  - `scripts/treasury/treasury_disbursement_contract.py`
- Contract lane entrypoint:
  - `scripts/treasury/run_treasury_disbursement_contract_lane.sh`
- Deterministic fixture cases:
  - `fixtures/treasury_disbursement/approval_threshold_cases.json`

## Treasury Disbursement Evidence Contract

- Evidence bundle generator:
  - `bash scripts/treasury/generate_treasury_disbursement_evidence_bundle.sh --output-file /tmp/treasury-disbursement.json --disbursement-id disbursement-go-001 --treasury-account-id treasury-main-001 --destination-account-id ops-wallet-001 --asset-symbol KAMN --disbursement-amount 250000 --daily-limit-amount 500000 --required-approvals 2 --received-approvals 2 --approval-quorum-hash sha256:approval-go-001 --policy-window-open true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/treasury/check_treasury_disbursement_policy.sh --bundle-file /tmp/treasury-disbursement.json`
- PR fast contract lane:
  - `bash scripts/treasury/run_treasury_disbursement_contract_lane.sh`
- Regression policy:
  - approval quorum shortfall, limit breaches, closed policy window, and tampered
    final decision force `NO-GO` (`Regression: #716`).

## CI Cost Strategy

- Treasury contract lane runs only for treasury-related docs/scripts/fixtures and
  shared framework helper changes.
- Runtime budget guard remains capped at 90 seconds in the contract lane.

## Local Validation

Run from repository root:

```bash
bash scripts/framework/test_contract_framework.sh
bash scripts/treasury/test_generate_treasury_disbursement_evidence_bundle.sh
bash scripts/treasury/test_run_treasury_disbursement_contract_lane.sh
bash scripts/ci/test_select_targets.sh
bash scripts/ci/test_ci_tools.sh
cargo fmt --check
cargo clippy -- -D warnings
```

