use super::support::assert_checklist_contains_all;

const REGRESSION_REQUIRES_STAGING_REHEARSAL_MISMATCH_GUARD_MARKERS: &[&str] = &[
    "rollback target hash mismatch and incomplete rehearsal evidence force `NO-GO` (`Regression: #623`).",
];

#[test]
fn regression_requires_staging_rehearsal_mismatch_guard() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_STAGING_REHEARSAL_MISMATCH_GUARD_MARKERS,
        "regression_requires_staging_rehearsal_mismatch_guard",
    );
}

const REGRESSION_REQUIRES_CHAIN_RECEIPT_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "missing or invalid chain receipt evidence forces `NO-GO` (`Regression: #678`).",
    "timeout-before-finality pending receipts and failed receipts force `NO-GO` (`Regression: #678`).",
];

#[test]
fn regression_requires_chain_receipt_evidence_guard_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_CHAIN_RECEIPT_EVIDENCE_GUARD_MARKER_MARKERS,
        "regression_requires_chain_receipt_evidence_guard_marker",
    );
}

const REGRESSION_REQUIRES_LEDGER_REFERENCE_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "missing ledger reference evidence and ledger amount drift force `NO-GO` (`Regression: #717`).",
];

#[test]
fn regression_requires_ledger_reference_evidence_guard_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_LEDGER_REFERENCE_EVIDENCE_GUARD_MARKER_MARKERS,
        "regression_requires_ledger_reference_evidence_guard_marker",
    );
}

const REGRESSION_REQUIRES_DURABLE_GUARD_SHARED_CONTRACT_MARKER_MARKERS: &[&str] = &[
    "shared contract-lane module marker remains required for docs/contracts drift guard (`Regression: #1242`).",
];

#[test]
fn regression_requires_durable_guard_shared_contract_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_DURABLE_GUARD_SHARED_CONTRACT_MARKER_MARKERS,
        "regression_requires_durable_guard_shared_contract_marker",
    );
}

const REGRESSION_REQUIRES_FAILOVER_SYNC_BUDGET_AND_CADENCE_GUARD_MARKERS_MARKERS: &[&str] = &[
    "preflight runtime budget overruns force lane failure (`Regression: #788`).",
    "unscheduled deep-lane execution force-fails via scheduled-only cadence guard (`Regression: #788`).",
];

#[test]
fn regression_requires_failover_sync_budget_and_cadence_guard_markers() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_FAILOVER_SYNC_BUDGET_AND_CADENCE_GUARD_MARKERS_MARKERS,
        "regression_requires_failover_sync_budget_and_cadence_guard_markers",
    );
}

const REGRESSION_REQUIRES_LIVE_NETWORK_PILOT_LAUNCH_AND_ROLLBACK_GUARD_MARKER_MARKERS: &[&str] = &[
    "missing smoke/deep pilot evidence or non-`GO` pilot decisions force launch `NO-GO` and trigger rollback review (`Regression: #830`).",
];

#[test]
fn regression_requires_live_network_pilot_launch_and_rollback_guard_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_LIVE_NETWORK_PILOT_LAUNCH_AND_ROLLBACK_GUARD_MARKER_MARKERS,
        "regression_requires_live_network_pilot_launch_and_rollback_guard_marker",
    );
}

const REGRESSION_REQUIRES_LIVE_NETWORK_PARTITION_RECONNECT_GUARD_MARKER_MARKERS: &[&str] = &[
    "stale/tampered partition/reconnect matrix artifacts and replay anomalies force `NO-GO` (`Regression: #982`).",
];

#[test]
fn regression_requires_live_network_partition_reconnect_guard_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_LIVE_NETWORK_PARTITION_RECONNECT_GUARD_MARKER_MARKERS,
        "regression_requires_live_network_partition_reconnect_guard_marker",
    );
}

const REGRESSION_REQUIRES_WATCHDOG_PROOF_CONSENSUS_BUDGET_AND_CADENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "proof-consensus deep-lane budget overruns and unscheduled cadence execution force `NO-GO` (`Regression: #996`).",
];

#[test]
fn regression_requires_watchdog_proof_consensus_budget_and_cadence_guard_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_WATCHDOG_PROOF_CONSENSUS_BUDGET_AND_CADENCE_GUARD_MARKER_MARKERS,
        "regression_requires_watchdog_proof_consensus_budget_and_cadence_guard_marker",
    );
}

const REGRESSION_REQUIRES_SOC2_CONTROL_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "tampered final decisions and incomplete/tampered control evidence force `NO-GO` (`Regression: #732`).",
];

#[test]
fn regression_requires_soc2_control_evidence_guard_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_SOC2_CONTROL_EVIDENCE_GUARD_MARKER_MARKERS,
        "regression_requires_soc2_control_evidence_guard_marker",
    );
}

const REGRESSION_REQUIRES_DSAR_LEGAL_HOLD_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] =
    &["legal-hold bypass attempts and tampered DSAR evidence force `NO-GO` (`Regression: #732`)."];

#[test]
fn regression_requires_dsar_legal_hold_evidence_guard_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_DSAR_LEGAL_HOLD_EVIDENCE_GUARD_MARKER_MARKERS,
        "regression_requires_dsar_legal_hold_evidence_guard_marker",
    );
}

const REGRESSION_REQUIRES_FEDERATED_DID_HANDSHAKE_EVIDENCE_GUARD_MARKER_MARKERS: &[&str] = &[
    "replay/downgrade attempts, quorum shortfalls, and tampered final decisions force `NO-GO` (`Regression: #734`).",
];

#[test]
fn regression_requires_federated_did_handshake_evidence_guard_marker() {
    assert_checklist_contains_all(
        REGRESSION_REQUIRES_FEDERATED_DID_HANDSHAKE_EVIDENCE_GUARD_MARKER_MARKERS,
        "regression_requires_federated_did_handshake_evidence_guard_marker",
    );
}
