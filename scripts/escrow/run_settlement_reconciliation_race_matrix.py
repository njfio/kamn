#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class MatrixCase:
    name: str
    escrow_id: str
    settlement_outcome: str
    receipt_id: str
    receipt_finality: str
    expected_release_amount: int
    expected_refund_amount: int
    observed_release_amount: int
    observed_refund_amount: int
    ledger_reference_id: str
    timeout_elapsed: bool
    ci_fast_gate: str
    expected_decision: str


def _run_script(command: list[str]) -> dict[str, str]:
    completed = subprocess.run(command, check=True, text=True, capture_output=True)
    pairs: dict[str, str] = {}
    for line in completed.stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        pairs[key.strip()] = value.strip()
    return pairs


def _load_cases(fixture_file: Path) -> list[MatrixCase]:
    payload = json.loads(fixture_file.read_text(encoding="utf-8"))
    if payload.get("schema_version") != "kamn.escrow.settlement-race-fixtures.v1":
        raise ValueError("unexpected fixture schema_version for settlement race matrix")

    cases_payload = payload.get("cases")
    if not isinstance(cases_payload, list) or not cases_payload:
        raise ValueError("settlement race fixture must contain a non-empty cases array")

    cases: list[MatrixCase] = []
    for item in cases_payload:
        if not isinstance(item, dict):
            raise ValueError("each settlement race fixture case must be an object")
        case = MatrixCase(
            name=str(item["name"]),
            escrow_id=str(item["escrow_id"]),
            settlement_outcome=str(item["settlement_outcome"]),
            receipt_id=str(item["receipt_id"]),
            receipt_finality=str(item["receipt_finality"]),
            expected_release_amount=int(item["expected_release_amount"]),
            expected_refund_amount=int(item["expected_refund_amount"]),
            observed_release_amount=int(item["observed_release_amount"]),
            observed_refund_amount=int(item["observed_refund_amount"]),
            ledger_reference_id=str(item["ledger_reference_id"]),
            timeout_elapsed=bool(item["timeout_elapsed"]),
            ci_fast_gate=str(item["ci_fast_gate"]),
            expected_decision=str(item["expected_decision"]),
        )
        cases.append(case)
    return cases


def _build_report_case(name: str, expected: str, generated: str, policy: str, passed: bool) -> dict[str, Any]:
    return {
        "name": name,
        "expected_decision": expected,
        "generated_decision": generated,
        "policy_decision": policy,
        "passed": passed,
    }


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run settlement reconciliation timeout/finality race matrix fixtures."
    )
    parser.add_argument("--fixture", required=True, help="Fixture JSON file path")
    parser.add_argument("--output-json", required=True, help="Output report JSON path")
    args = parser.parse_args()

    root_dir = Path(__file__).resolve().parents[2]
    fixture_file = Path(args.fixture).resolve()
    output_file = Path(args.output_json).resolve()
    output_file.parent.mkdir(parents=True, exist_ok=True)

    generator = root_dir / "scripts/escrow/generate_settlement_reconciliation_evidence_bundle.sh"
    checker = root_dir / "scripts/escrow/check_settlement_reconciliation_evidence_policy.sh"
    if not generator.is_file() or not checker.is_file():
        raise FileNotFoundError("settlement reconciliation generator/checker scripts are required")

    cases = _load_cases(fixture_file)

    report_cases: list[dict[str, Any]] = []
    failed_count = 0

    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp_root = Path(tmp_dir)
        for case in cases:
            case_bundle = tmp_root / f"{case.name}.json"
            generated_output = _run_script(
                [
                    "bash",
                    str(generator),
                    "--output-file",
                    str(case_bundle),
                    "--escrow-id",
                    case.escrow_id,
                    "--settlement-outcome",
                    case.settlement_outcome,
                    "--receipt-id",
                    case.receipt_id,
                    "--receipt-finality",
                    case.receipt_finality,
                    "--expected-release-amount",
                    str(case.expected_release_amount),
                    "--expected-refund-amount",
                    str(case.expected_refund_amount),
                    "--observed-release-amount",
                    str(case.observed_release_amount),
                    "--observed-refund-amount",
                    str(case.observed_refund_amount),
                    "--ledger-reference-id",
                    case.ledger_reference_id,
                    "--timeout-elapsed",
                    "true" if case.timeout_elapsed else "false",
                    "--ci-fast-gate",
                    case.ci_fast_gate,
                ]
            )
            policy_output = _run_script(
                [
                    "bash",
                    str(checker),
                    "--bundle-file",
                    str(case_bundle),
                ]
            )

            generated_decision = generated_output.get("final_decision", "")
            policy_decision = policy_output.get("final_decision", "")
            passed = (
                generated_decision == case.expected_decision
                and policy_decision == case.expected_decision
            )
            if not passed:
                failed_count += 1

            report_cases.append(
                _build_report_case(
                    case.name,
                    case.expected_decision,
                    generated_decision,
                    policy_decision,
                    passed,
                )
            )

    report = {
        "schema_version": "kamn.escrow.settlement-race-matrix.v1",
        "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "fixture": str(fixture_file),
        "case_count": len(report_cases),
        "failed_count": failed_count,
        "cases": report_cases,
    }
    output_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    if failed_count > 0:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
