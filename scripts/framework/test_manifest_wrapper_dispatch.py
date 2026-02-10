#!/usr/bin/env python3
"""Tests for manifest lane wrapper dispatch behavior."""

from __future__ import annotations

import json
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT_DIR = Path(__file__).resolve().parents[2]
WRAPPER_SCRIPT = ROOT_DIR / "scripts/framework/run_manifest_lane.sh"


class ManifestWrapperDispatchTests(unittest.TestCase):
    def _write_manifest(self, path: Path) -> None:
        path.write_text(
            json.dumps(
                {
                    "schema_version": "kamn.contract-lane.manifest.v1",
                    "lane_id": "framework.dispatch",
                    "evidence_key": "framework_dispatch:v1",
                    "reason_key": "framework_dispatch_reason_codes:GO:v1",
                    "phases": {
                        "generate": ["python3", "-c", "print('dispatch-ok')"],
                    },
                }
            ),
            encoding="utf-8",
        )

    def test_wrapper_executes_manifest_phase(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            manifest_path = Path(temp_dir) / "manifest.json"
            self._write_manifest(manifest_path)

            completed = subprocess.run(
                ["bash", str(WRAPPER_SCRIPT), "--manifest", str(manifest_path), "--phase", "generate"],
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertEqual(completed.returncode, 0)
        self.assertIn("status=ok", completed.stdout)
        self.assertIn("lane_id=framework.dispatch", completed.stdout)
        self.assertIn("dispatch-ok", completed.stdout)

    def test_wrapper_fails_for_unknown_phase(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            manifest_path = Path(temp_dir) / "manifest.json"
            self._write_manifest(manifest_path)

            completed = subprocess.run(
                ["bash", str(WRAPPER_SCRIPT), "--manifest", str(manifest_path), "--phase", "missing"],
                capture_output=True,
                text=True,
                check=False,
            )

        self.assertNotEqual(completed.returncode, 0)
        self.assertIn("status=fail", completed.stdout + completed.stderr)


if __name__ == "__main__":
    unittest.main()
