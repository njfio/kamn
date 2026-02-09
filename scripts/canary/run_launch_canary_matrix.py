#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any


@dataclass(frozen=True)
class ProbeCase:
    name: str
    message_probe: str
    task_probe: str
    escrow_probe: str
    evidence_complete: bool
    latency_budget_ms: int
    observed_latency_ms: int
    expected_decision: str


def _load_cases(fixture_file: Path) -> list[ProbeCase]:
    payload = json.loads(fixture_file.read_text(encoding="utf-8"))
    if payload.get("schema_version") != "kamn.launch-canary.probe-fixtures.v1":
        raise ValueError("unexpected launch canary fixture schema_version")

    cases_payload = payload.get("cases")
    if not isinstance(cases_payload, list) or not cases_payload:
        raise ValueError("launch canary fixture must contain a non-empty cases array")

    cases: list[ProbeCase] = []
    for item in cases_payload:
        if not isinstance(item, dict):
            raise ValueError("each launch canary case must be an object")
        case = ProbeCase(
            name=str(item["name"]),
            message_probe=str(item["message_probe"]),
            task_probe=str(item["task_probe"]),
            escrow_probe=str(item["escrow_probe"]),
            evidence_complete=bool(item["evidence_complete"]),
            latency_budget_ms=int(item["latency_budget_ms"]),
            observed_latency_ms=int(item["observed_latency_ms"]),
            expected_decision=str(item["expected_decision"]),
        )
        cases.append(case)
    return cases


def _evaluate_case(case: ProbeCase) -> tuple[str, list[str]]:
    reasons: list[str] = []
    if case.message_probe != "PASS":
        reasons.append("message-probe-failed")
    if case.task_probe != "PASS":
        reasons.append("task-probe-failed")
    if case.escrow_probe != "PASS":
        reasons.append("escrow-probe-failed")
    if not case.evidence_complete:
        reasons.append("missing-probe-evidence")
    if case.observed_latency_ms > case.latency_budget_ms:
        reasons.append("latency-budget-exceeded")

    decision = "GO" if not reasons else "NO-GO"
    return decision, reasons


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Run launch canary probe matrix fixtures and emit deterministic report output."
    )
    parser.add_argument("--fixture", required=True, help="Fixture JSON file path")
    parser.add_argument("--output-json", required=True, help="Output report JSON path")
    args = parser.parse_args()

    fixture_file = Path(args.fixture).resolve()
    report_file = Path(args.output_json).resolve()
    report_file.parent.mkdir(parents=True, exist_ok=True)

    if not fixture_file.is_file():
        raise FileNotFoundError(f"fixture file not found: {fixture_file}")

    cases = _load_cases(fixture_file)
    report_cases: list[dict[str, Any]] = []
    failed_count = 0

    for case in cases:
        derived_decision, reasons = _evaluate_case(case)
        passed = derived_decision == case.expected_decision
        if not passed:
            failed_count += 1

        report_cases.append(
            {
                "name": case.name,
                "expected_decision": case.expected_decision,
                "derived_decision": derived_decision,
                "reasons": reasons,
                "passed": passed,
            }
        )

    report = {
        "schema_version": "kamn.launch-canary.probe-report.v1",
        "generated_at": datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "fixture": str(fixture_file),
        "case_count": len(report_cases),
        "failed_count": failed_count,
        "cases": report_cases,
    }
    report_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    print("status=completed")
    print(f"fixture={fixture_file}")
    print(f"case_count={len(report_cases)}")
    print(f"failed_count={failed_count}")
    print(f"output_json={report_file}")

    if failed_count > 0:
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
