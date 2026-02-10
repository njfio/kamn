#!/usr/bin/env python3
"""Unit tests for pilot dashboard/compliance lane manifests."""

from __future__ import annotations

from pathlib import Path
import unittest

from lane_manifest import MANIFEST_SCHEMA_VERSION, load_manifest_file

ROOT_DIR = Path(__file__).resolve().parents[2]
MANIFEST_DIR = ROOT_DIR / "scripts/framework/manifests"


class PilotLaneManifestTests(unittest.TestCase):
    def test_dashboard_and_compliance_manifests_parse_and_route(self) -> None:
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
