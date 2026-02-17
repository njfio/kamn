#!/usr/bin/env python3
"""Unit tests for declarative policy checker evaluation behavior."""

from __future__ import annotations

import json
from pathlib import Path
import tempfile
import unittest

from declarative_policy_checker import (  # noqa: E402
    OUTPUT_SCHEMA_VERSION,
    POLICY_SCHEMA_VERSION,
    _MISSING,
    _evaluate,
    _evaluate_check,
    _resolve_field,
    _validate_policy,
    main,
)
from framework.contract_framework import ContractError


class DeclarativePolicyCheckerTests(unittest.TestCase):
    def _base_policy(self) -> dict:
        return {
            "schema_version": POLICY_SCHEMA_VERSION,
            "policy_name": "example.policy",
            "reason_taxonomy_version": "kamn.example.reason-taxonomy.v1",
            "reason_key_prefix": "example_reason_codes",
            "success_reason_code": "none",
            "checks": [
                {
                    "field": "status",
                    "op": "equals",
                    "expected": "pass",
                    "reason_code": "status_not_pass",
                },
                {
                    "field": "error_count",
                    "op": "lte",
                    "expected": 0,
                    "reason_code": "error_count_exceeded",
                },
            ],
        }

    def _write_json(self, path: Path, payload: dict) -> None:
        path.write_text(json.dumps(payload), encoding="utf-8")

    def test_validate_policy_accepts_valid_payload(self) -> None:
        policy = _validate_policy(self._base_policy())
        self.assertEqual(policy["policy_name"], "example.policy")
        self.assertEqual(policy["schema_version"], POLICY_SCHEMA_VERSION)

    def test_validate_policy_rejects_missing_expected_for_non_exists_check(self) -> None:
        policy = self._base_policy()
        policy["checks"][0].pop("expected")

        with self.assertRaisesRegex(ContractError, "missing required 'expected' value"):
            _validate_policy(policy)

    def test_resolve_field_supports_nested_dict_and_list_lookup(self) -> None:
        payload = {"outer": {"items": [{"value": "ok"}]}}
        self.assertEqual(_resolve_field(payload, "outer.items.0.value"), "ok")
        self.assertIs(_resolve_field(payload, "outer.items.one.value"), _MISSING)
        self.assertIs(_resolve_field(payload, "outer.items.9.value"), _MISSING)

    def test_evaluate_check_supports_exists_and_regex(self) -> None:
        payload = {"status": "pass", "detail": "lane finished successfully"}
        self.assertTrue(
            _evaluate_check(
                {"field": "status", "op": "exists", "expected": True, "reason_code": "unused"},
                payload,
            )
        )
        self.assertTrue(
            _evaluate_check(
                {
                    "field": "detail",
                    "op": "regex",
                    "expected": "finished\\s+successfully",
                    "reason_code": "unused",
                },
                payload,
            )
        )
        self.assertFalse(
            _evaluate_check(
                {"field": "missing", "op": "exists", "expected": True, "reason_code": "unused"},
                payload,
            )
        )

    def test_evaluate_returns_go_with_success_reason(self) -> None:
        policy = _validate_policy(self._base_policy())
        report = {"status": "pass", "error_count": 0}

        result = _evaluate(policy, report)
        self.assertEqual(result["schema_version"], OUTPUT_SCHEMA_VERSION)
        self.assertEqual(result["status"], "pass")
        self.assertEqual(result["final_decision"], "GO")
        self.assertEqual(result["reason_codes"], ["none"])
        self.assertEqual(result["reason_key"], "example_reason_codes:GO:v1")
        self.assertEqual(result["failed_check_count"], 0)

    def test_evaluate_returns_no_go_with_all_failed_reason_codes(self) -> None:
        policy = _validate_policy(self._base_policy())
        report = {"status": "fail", "error_count": 3}

        result = _evaluate(policy, report)
        self.assertEqual(result["status"], "fail")
        self.assertEqual(result["final_decision"], "NO-GO")
        self.assertEqual(result["reason_codes"], ["status_not_pass", "error_count_exceeded"])
        self.assertEqual(result["reason_key"], "example_reason_codes:NO-GO:v1")
        self.assertEqual(result["failed_check_count"], 2)

    def test_main_writes_output_json_and_ci_fast_gate(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            policy_path = temp_path / "policy.json"
            report_path = temp_path / "report.json"
            output_path = temp_path / "output.json"
            self._write_json(policy_path, self._base_policy())
            self._write_json(report_path, {"status": "pass", "error_count": 0})

            rc = main(
                [
                    "--policy-file",
                    str(policy_path),
                    "--report-file",
                    str(report_path),
                    "--expected-final-decision",
                    "GO",
                    "--ci-fast-gate",
                    "PASS",
                    "--output-json",
                    str(output_path),
                ]
            )

            self.assertEqual(rc, 0)
            output = json.loads(output_path.read_text(encoding="utf-8"))
            self.assertEqual(output["final_decision"], "GO")
            self.assertEqual(output["ci_fast_gate"], "PASS")
            self.assertEqual(output["policy_name"], "example.policy")

    def test_main_raises_when_expected_final_decision_mismatches(self) -> None:
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            policy_path = temp_path / "policy.json"
            report_path = temp_path / "report.json"
            self._write_json(policy_path, self._base_policy())
            self._write_json(report_path, {"status": "fail", "error_count": 3})

            with self.assertRaisesRegex(ContractError, "expected-final-decision mismatch"):
                main(
                    [
                        "--policy-file",
                        str(policy_path),
                        "--report-file",
                        str(report_path),
                        "--expected-final-decision",
                        "GO",
                    ]
                )


if __name__ == "__main__":
    unittest.main()
