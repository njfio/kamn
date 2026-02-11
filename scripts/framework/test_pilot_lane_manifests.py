#!/usr/bin/env python3
"""Unit tests for pilot lane manifests."""

from __future__ import annotations

from pathlib import Path
import unittest

from lane_manifest import MANIFEST_SCHEMA_VERSION, load_manifest_file

ROOT_DIR = Path(__file__).resolve().parents[2]
MANIFEST_DIR = ROOT_DIR / "scripts/framework/manifests"


class PilotLaneManifestTests(unittest.TestCase):
    def test_pilot_manifests_parse_and_route(self) -> None:
        cases = (
            (
                "dashboard_backend_session_auth_freshness_contract_lane.json",
                "dashboard.backend_session_auth_freshness.contract",
                "scripts/dashboard/backend_session_auth_freshness_contract_lane_contract.py",
            ),
            (
                "dashboard_stale_error_budget_contract_lane.json",
                "dashboard.stale_error_budget.contract",
                "scripts/dashboard/stale_error_budget_contract_lane_contract.py",
            ),
            (
                "compliance_soc2_control_evidence_contract_lane.json",
                "compliance.soc2_control_evidence.contract",
                "scripts/compliance/soc2_control_evidence_contract_lane_contract.py",
            ),
            (
                "compliance_dsar_legal_hold_contract_lane.json",
                "compliance.dsar_legal_hold.contract",
                "scripts/compliance/dsar_legal_hold_contract_lane_contract.py",
            ),
            (
                "compliance_classification_redaction_contract_lane.json",
                "compliance.classification_redaction.contract",
                "scripts/compliance/classification_redaction_contract_lane_contract.py",
            ),
            (
                "governance_simulation_contract_lane.json",
                "governance.simulation.contract",
                "scripts/governance/governance_simulation_contract_lane_contract.py",
            ),
            (
                "governance_lifecycle_rollback_contract_lane.json",
                "governance.lifecycle_rollback.contract",
                "scripts/governance/governance_lifecycle_rollback_contract_lane_contract.py",
            ),
            (
                "governance_quorum_attestation_replay_contract_lane.json",
                "governance.quorum_attestation_replay.contract",
                "scripts/governance/governance_quorum_attestation_replay_contract_lane_contract.py",
            ),
            (
                "governance_stake_slash_risk_contract_lane.json",
                "governance.stake_slash_risk.contract",
                "scripts/governance/stake_slash_risk_contract_lane_contract.py",
            ),
            (
                "reputation_dispute_contract_lane.json",
                "reputation.dispute.contract",
                "scripts/reputation/reputation_dispute_contract_lane_contract.py",
            ),
            (
                "canary_launch_canary_contract_lane.json",
                "canary.launch_canary.contract",
                "scripts/canary/launch_canary_contract_lane_contract.py",
            ),
            (
                "canary_post_cutover_slo_contract_lane.json",
                "canary.post_cutover_slo.contract",
                "scripts/canary/post_cutover_slo_contract_lane_contract.py",
            ),
            (
                "token_launch_handoff_contract_lane.json",
                "token.launch_handoff.contract",
                "scripts/token/token_launch_handoff_contract_lane_contract.py",
            ),
            (
                "treasury_disbursement_contract_lane.json",
                "treasury.disbursement.contract",
                "scripts/treasury/treasury_disbursement_contract_lane_contract.py",
            ),
            (
                "guard_durable_guard_recovery_contract_lane.json",
                "guard.durable_guard_recovery.contract",
                "scripts/guard/durable_guard_recovery_contract_lane_contract.py",
            ),
            (
                "kolme_snapshot_drift_contract_lane.json",
                "kolme.snapshot_drift.contract",
                "scripts/kolme/contracts/snapshot_drift_contract_lane.py",
            ),
            (
                "kolme_notifications_consumer_contract_lane.json",
                "kolme.notifications.consumer.contract",
                "scripts/kolme/contracts/notifications_consumer_contract_lane.py",
            ),
            (
                "kolme_block_fallback_reconciliation_contract_lane.json",
                "kolme.block_fallback.reconciliation.contract",
                "scripts/kolme/contracts/block_fallback_reconciliation_contract_lane.py",
            ),
        )

        for manifest_name, expected_lane_id, expected_script in cases:
            manifest = load_manifest_file(MANIFEST_DIR / manifest_name)
            self.assertEqual(manifest.schema_version, MANIFEST_SCHEMA_VERSION)
            self.assertEqual(manifest.lane_id, expected_lane_id)
            self.assertIn("contract", manifest.phases)
            self.assertEqual(
                manifest.phases["contract"],
                ("python3", expected_script),
            )


if __name__ == "__main__":
    unittest.main()
