#!/usr/bin/env python3
"""Unit tests for shared contract framework helpers."""

from __future__ import annotations

import pathlib
import tempfile
import unittest
import sys

ROOT_DIR = pathlib.Path(__file__).resolve().parents[2]
sys.path.insert(0, str(ROOT_DIR / "scripts"))

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    load_json,
    require_non_negative_int,
    require_pattern,
    write_json,
)


class ContractFrameworkTests(unittest.TestCase):
    def test_non_negative_int_parser_accepts_zero(self) -> None:
        self.assertEqual(require_non_negative_int("value", "0"), 0)

    def test_non_negative_int_parser_rejects_negative(self) -> None:
        with self.assertRaisesRegex(ContractError, "must be >= 0"):
            require_non_negative_int("value", "-1")

    def test_pattern_validator_rejects_invalid_symbol(self) -> None:
        with self.assertRaisesRegex(ContractError, "uppercase"):
            require_pattern(
                "token_symbol",
                "kamn",
                r"[A-Z0-9]+",
                "token_symbol must be uppercase alphanumeric",
            )

    def test_decision_accumulator_reports_no_go_when_reason_present(self) -> None:
        decision = DecisionAccumulator()
        decision.reject_if(True, "example failure")
        final_decision, reasons = decision.finalize("all good")
        self.assertEqual(final_decision, "NO-GO")
        self.assertEqual(reasons, ["example failure"])

    def test_json_round_trip_preserves_payload(self) -> None:
        with tempfile.TemporaryDirectory() as tmp_dir:
            output_path = pathlib.Path(tmp_dir) / "payload.json"
            payload = {"schema_version": "example.v1", "final_decision": "GO"}
            write_json(output_path, payload)
            parsed = load_json(output_path)
            self.assertEqual(parsed, payload)


if __name__ == "__main__":
    unittest.main()

