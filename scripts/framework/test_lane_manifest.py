#!/usr/bin/env python3
"""Unit tests for lane manifest parsing and runner helpers."""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import unittest

from lane_manifest import (
    MANIFEST_SCHEMA_VERSION,
    LaneManifest,
    load_manifest_file,
    parse_manifest,
    run_lane_phase,
)


class LaneManifestTests(unittest.TestCase):
    def test_parse_manifest_accepts_valid_payload(self) -> None:
        manifest = parse_manifest(
            {
                "schema_version": MANIFEST_SCHEMA_VERSION,
                "lane_id": "dashboard.shell.matrix",
                "evidence_key": "frontend_shell_matrix:v1",
                "reason_key": "frontend_shell_matrix_reason_codes:GO:v1",
                "phases": {
                    "generate": ["python3", "-c", "print('generate')"],
                    "check": ["python3", "-c", "print('check')"],
                },
            }
        )

        self.assertIsInstance(manifest, LaneManifest)
        self.assertEqual(manifest.lane_id, "dashboard.shell.matrix")
        self.assertIn("generate", manifest.phases)
        self.assertIn("check", manifest.phases)

    def test_parse_manifest_rejects_missing_required_fields(self) -> None:
        with self.assertRaises(ValueError):
            parse_manifest(
                {
                    "schema_version": MANIFEST_SCHEMA_VERSION,
                    "lane_id": "dashboard.shell.matrix",
                    "evidence_key": "frontend_shell_matrix:v1",
                    "phases": {"generate": ["python3", "-c", "print('generate')"]},
                }
            )

    def test_load_manifest_file_parses_json(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            manifest_path = Path(temp_dir) / "lane-manifest.json"
            manifest_path.write_text(
                json.dumps(
                    {
                        "schema_version": MANIFEST_SCHEMA_VERSION,
                        "lane_id": "governance.simulation",
                        "evidence_key": "governance_simulation:v1",
                        "reason_key": "governance_simulation_reason_codes:GO:v1",
                        "phases": {
                            "run": ["python3", "-c", "print('run')"],
                        },
                    }
                ),
                encoding="utf-8",
            )
            manifest = load_manifest_file(manifest_path)

        self.assertEqual(manifest.lane_id, "governance.simulation")
        self.assertIn("run", manifest.phases)

    def test_run_lane_phase_executes_phase_command(self) -> None:
        manifest = parse_manifest(
            {
                "schema_version": MANIFEST_SCHEMA_VERSION,
                "lane_id": "contract.framework",
                "evidence_key": "contract_framework:v1",
                "reason_key": "contract_framework_reason_codes:GO:v1",
                "phases": {
                    "test": ["python3", "-c", "print('manifest-phase-ok')"],
                },
            }
        )
        code, output = run_lane_phase(manifest, "test")

        self.assertEqual(code, 0)
        self.assertIn("manifest-phase-ok", output)

    def test_run_lane_phase_rejects_unknown_phase(self) -> None:
        manifest = parse_manifest(
            {
                "schema_version": MANIFEST_SCHEMA_VERSION,
                "lane_id": "contract.framework",
                "evidence_key": "contract_framework:v1",
                "reason_key": "contract_framework_reason_codes:GO:v1",
                "phases": {
                    "generate": ["python3", "-c", "print('ok')"],
                },
            }
        )

        with self.assertRaises(ValueError):
            run_lane_phase(manifest, "missing-phase")

    def test_run_lane_phase_forwards_phase_args(self) -> None:
        manifest = parse_manifest(
            {
                "schema_version": MANIFEST_SCHEMA_VERSION,
                "lane_id": "contract.framework",
                "evidence_key": "contract_framework:v1",
                "reason_key": "contract_framework_reason_codes:GO:v1",
                "phases": {
                    "test": [
                        "python3",
                        "-c",
                        "import sys; print('args=' + ' '.join(sys.argv[1:]))",
                    ],
                },
            }
        )
        code, output = run_lane_phase(
            manifest,
            "test",
            phase_args=["--output-file", "report.json"],
        )

        self.assertEqual(code, 0)
        self.assertIn("args=--output-file report.json", output)


if __name__ == "__main__":
    unittest.main()
