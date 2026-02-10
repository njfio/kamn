#!/usr/bin/env python3
"""Unit tests for shared contract-lane helper utilities."""

from __future__ import annotations

import unittest

from contract_lane_helpers import build_default_bundle_args, run_capture


class ContractLaneHelpersTests(unittest.TestCase):
    def test_run_capture_returns_exit_code_and_output(self) -> None:
        code, output = run_capture(["python3", "-c", "print('ok')"])
        self.assertEqual(code, 0)
        self.assertIn("ok", output)

    def test_build_default_bundle_args_contains_required_pairs(self) -> None:
        args = build_default_bundle_args(
            output_file="/tmp/example.json",
            pairs=(
                ("--control-id", "CC6.1"),
                ("--ci-fast-gate", "PASS"),
            ),
        )
        self.assertIn("--output-file", args)
        self.assertIn("/tmp/example.json", args)
        self.assertIn("--control-id", args)
        self.assertIn("CC6.1", args)


if __name__ == "__main__":
    unittest.main()
