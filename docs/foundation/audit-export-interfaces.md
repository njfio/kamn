# Compliance Audit Export Interfaces Slice (Issues #152, #153)

This document describes the first implementation slice for multi-domain compliance audit exports.

## Scope Delivered
- Added `AuditExportEngine` in `kamn-core` with deterministic export behavior across domains:
  - `messages`
  - `tasks`
  - `escrows`
  - `reputation`
- Added structured export contract:
  - `AuditEventRecord`
  - `AuditExportRequest`
  - `AuditExportFilter`
  - `AuditExportBundle`
  - `AuditExportManifest`
- Enforced export access controls:
  - Only authorized exporter DIDs can generate exports.
- Implemented deterministic filtering and ordering:
  - Domain filter
  - Actor allowlist filter
  - Inclusive time window filter
- Added integrity metadata:
  - Canonical export payload serialization
  - Deterministic `fnv1a64` hash in manifest

## Escrow-Ledger Reconciliation Evidence Contract (Issue #717)
Escrow settlement accounting must include deterministic ledger-reference evidence before reconciliation gates can approve release.

- Evidence bundle generator:
  - `bash scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh --output-file /tmp/settlement-evidence.json --escrow-id escrow-001 --settlement-outcome RELEASED --receipt-id receipt-001 --receipt-finality FINAL --expected-release-amount 120 --expected-refund-amount 0 --observed-release-amount 120 --observed-refund-amount 0 --ledger-reference-id ledger-entry-001 --timeout-elapsed false --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/escrow/check_settlement_reconciliation_evidence_policy.sh --bundle-file /tmp/settlement-evidence.json`
- PR fast contract lane:
  - `bash scripts/escrow/run_settlement_reconciliation_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/escrow/run_settlement_reconciliation_deep_lane.sh --output-json settlement-reconciliation-report.json`
- Regression policy:
  - missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`).

## SOC2 Control Evidence Bundle Contract (Issue #744)
SOC2 audit readiness requires deterministic control evidence bundles with replay-safe policy verification.

- Stable shell wrappers:
  - `scripts/compliance/generate_soc2_control_evidence_bundle.sh`
  - `scripts/compliance/check_soc2_control_evidence_policy.sh`
- Shared Python implementation:
  - `scripts/compliance/soc2_control_contract.py`
- Evidence bundle generator:
  - `bash scripts/compliance/generate_soc2_control_evidence_bundle.sh --output-file /tmp/soc2-control-evidence.json --control-id CC6.1 --audit-period-start 2026-01-01 --audit-period-end 2026-01-31 --collector-did did:kamn:auditor-001 --evidence-uri s3://kamn-audit/soc2/cc6_1/jan-2026/evidence.json --evidence-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --tamper-check PASS --completeness-check PASS --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/compliance/check_soc2_control_evidence_policy.sh --bundle-file /tmp/soc2-control-evidence.json`
- PR fast contract lane:
  - `bash scripts/compliance/run_soc2_control_evidence_contract_lane.sh`
- Shared contract-lane module:
  - `scripts/compliance/soc2_control_evidence_contract_lane_contract.py`
- Scheduled deep lane entrypoint:
  - `bash scripts/compliance/run_soc2_control_evidence_deep_lane.sh --output-json soc2-control-evidence-report.json`
- Replay matrix runner:
  - `python3 scripts/compliance/run_soc2_control_evidence_replay_matrix.py --fixture fixtures/compliance_soc2/control_evidence_replay_cases.json --output-json soc2-control-evidence-report.json`
- Regression policy:
  - tampered final decisions and incomplete/tampered control evidence force `NO-GO` (`Regression: #732`).
  - the shell contract-lane wrapper delegates orchestration logic to `soc2_control_evidence_contract_lane_contract.py` (`Regression: #1238`).

## DSAR Legal-Hold Evidence Contract (Issue #746)
Data-subject access/export/erasure evidence must enforce legal-hold precedence through deterministic bundle policy checks.

- Stable shell wrappers:
  - `scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh`
  - `scripts/compliance/check_dsar_legal_hold_policy.sh`
- Shared Python implementation:
  - `scripts/compliance/dsar_legal_hold_contract.py`
- Evidence bundle generator:
  - `bash scripts/compliance/generate_dsar_legal_hold_evidence_bundle.sh --output-file /tmp/dsar-legal-hold.json --request-id dsar-erasure-001 --subject-did did:kamn:subject-001 --request-type ERASURE --legal-hold-active true --retention-expired true --evidence-complete true --approval-recorded true --tamper-check PASS --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/compliance/check_dsar_legal_hold_policy.sh --bundle-file /tmp/dsar-legal-hold.json`
- PR fast contract lane:
  - `bash scripts/compliance/run_dsar_legal_hold_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/compliance/run_dsar_legal_hold_deep_lane.sh --output-json dsar-legal-hold-report.json`
- Replay matrix runner:
  - `python3 scripts/compliance/run_dsar_legal_hold_matrix.py --fixture fixtures/compliance_dsar/legal_hold_precedence_cases.json --output-json dsar-legal-hold-report.json`
- Regression policy:
  - legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`).

## Reputation Dispute Evidence Export Contract (Issue #737 / #738)
Reputation dispute exports require deterministic evidence bundles and checker outputs before score overrides are accepted.

- Stable shell wrappers:
  - `scripts/reputation/generate_reputation_dispute_evidence_bundle.sh`
  - `scripts/reputation/check_reputation_dispute_policy.sh`
- Shared Python implementation:
  - `scripts/reputation/reputation_dispute_contract.py`
- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_dispute_evidence_bundle.sh --output-file /tmp/reputation-dispute.json --dispute-id dispute-001 --subject-did did:kamn:agent-001 --reviewer-did did:kamn:reviewer-001 --dispute-reason-code QUALITY --evidence-uri s3://kamn-audit/reputation/dispute-001.json --evidence-sha256 sha256:1111111111111111111111111111111111111111111111111111111111111111 --evidence-hash-verified PASS --original-trust-score 640 --proposed-trust-score 560 --max-adjustment-points 120 --policy-window-open true --approval-recorded true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_dispute_policy.sh --bundle-file /tmp/reputation-dispute.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_dispute_contract_lane.sh`
- Scheduled deep lane entrypoint:
  - `bash scripts/reputation/run_reputation_dispute_deep_lane.sh --output-json reputation-dispute-report.json`
- Replay matrix runner:
  - `python3 scripts/reputation/run_reputation_dispute_matrix.py --fixture fixtures/reputation_dispute/replay_cases.json --output-json reputation-dispute-report.json`
- Regression policy:
  - tampered evidence hashes, score-adjustment limit bypasses, and closed-policy-window decisions force `NO-GO` (`Regression: #730`).

## Reputation Signal Quarantine Evidence Export Contract (Issue #935)
Inbound reputation signal exports require deterministic quarantine-policy evidence before ingestion actions are accepted.

- Stable shell wrappers:
  - `scripts/reputation/generate_reputation_signal_quarantine_evidence_bundle.sh`
  - `scripts/reputation/check_reputation_signal_quarantine_policy.sh`
- Shared Python implementation:
  - `scripts/reputation/reputation_signal_quarantine_contract.py`
- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_signal_quarantine_evidence_bundle.sh --output-file /tmp/reputation-signal-quarantine.json --lane contract --signal-id signal-001 --subject-did did:kamn:agent-001 --signal-kind ENDORSEMENT --source-channel TELEGRAM --event-age-seconds 45 --payload-sha256 sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa --payload-signature-verified PASS --nonce-unique true --rate-within-threshold true --source-attested true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_signal_quarantine_policy.sh --bundle-file /tmp/reputation-signal-quarantine.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_signal_quarantine_contract_lane.sh`
- Regression policy:
  - tampered reason keys/reason codes and ingestion-action mismatches force `NO-GO` (`Regression: #935`).

## Reputation Recovery Reversal Evidence Export Contract (Issue #936)
False-positive penalty recovery exports require deterministic reversal-policy evidence before remediation actions are accepted.

- Stable shell wrappers:
  - `scripts/reputation/generate_reputation_recovery_evidence_bundle.sh`
  - `scripts/reputation/check_reputation_recovery_policy.sh`
- Shared Python implementation:
  - `scripts/reputation/reputation_recovery_contract.py`
- Evidence bundle generator:
  - `bash scripts/reputation/generate_reputation_recovery_evidence_bundle.sh --output-file /tmp/reputation-recovery.json --lane contract --recovery-id recovery-001 --subject-did did:kamn:agent-001 --reviewer-did did:kamn:reviewer-001 --pre-penalty-trust-score 700 --post-penalty-trust-score 540 --proposed-recovered-trust-score 660 --max-reversal-points 160 --false-positive-confirmed true --reviewer-quorum-satisfied true --audit-evidence-verified PASS --replay-guard-pass true --ci-fast-gate PASS`
- Policy checker:
  - `bash scripts/reputation/check_reputation_recovery_policy.sh --bundle-file /tmp/reputation-recovery.json`
- PR fast contract lane:
  - `bash scripts/reputation/run_reputation_recovery_contract_lane.sh`
- Regression policy:
  - false-positive irreversible-penalty paths, replayed recovery nonces, and tampered recovery reason codes force `NO-GO` (`Regression: #936`).

## Local Validation
Run from repository root:

```bash
bash scripts/compliance/test_generate_soc2_control_evidence_bundle.sh
bash scripts/compliance/test_run_soc2_control_evidence_contract_lane.sh
bash scripts/compliance/test_run_soc2_control_evidence_replay_matrix.sh
bash scripts/compliance/test_run_soc2_control_evidence_deep_lane.sh
bash scripts/compliance/test_generate_dsar_legal_hold_evidence_bundle.sh
bash scripts/compliance/test_run_dsar_legal_hold_contract_lane.sh
bash scripts/compliance/test_run_dsar_legal_hold_matrix.sh
bash scripts/compliance/test_run_dsar_legal_hold_deep_lane.sh
bash scripts/reputation/test_generate_reputation_dispute_evidence_bundle.sh
bash scripts/reputation/test_run_reputation_dispute_contract_lane.sh
bash scripts/reputation/test_run_reputation_dispute_matrix.sh
bash scripts/reputation/test_run_reputation_dispute_deep_lane.sh
bash scripts/reputation/test_generate_reputation_signal_quarantine_evidence_bundle.sh
bash scripts/reputation/test_check_reputation_signal_quarantine_policy.sh
bash scripts/reputation/test_run_reputation_signal_quarantine_contract_lane.sh
bash scripts/reputation/test_generate_reputation_recovery_evidence_bundle.sh
bash scripts/reputation/test_check_reputation_recovery_policy.sh
bash scripts/reputation/test_run_reputation_recovery_contract_lane.sh
cargo test -p kamn-core --test audit_export_interfaces
cargo test -p kamn-core --test audit_export_interfaces_docs
cargo fmt --check
cargo clippy -- -D warnings
cargo test -p kamn-core
```

## Follow-up
- Add explicit export scope permissions mapped to operator roles and tenant boundaries.
- Add stream-oriented export interfaces for large historical datasets.
