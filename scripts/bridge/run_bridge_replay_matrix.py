#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any, Dict, List, Tuple


ROOT_DIR = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fixture",
        default=str(ROOT_DIR / "fixtures/bridge_replay/replay_validation_cases.json"),
    )
    parser.add_argument(
        "--suites",
        default="",
        help="Comma-separated suite names to include (for example: telegram_bridge,bridge_adapter)",
    )
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def parse_key_values(stdout: str) -> Dict[str, str]:
    values: Dict[str, str] = {}
    for raw_line in stdout.splitlines():
        line = raw_line.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key] = value
    return values


def parse_requested_suites(raw: str) -> List[str]:
    requested: List[str] = []
    seen = set()
    for part in raw.split(","):
        suite = part.strip()
        if not suite:
            continue
        if suite in seen:
            continue
        seen.add(suite)
        requested.append(suite)
    return requested


def run_case(case: Dict[str, Any]) -> Dict[str, str]:
    command = [
        "bash",
        str(ROOT_DIR / "scripts/bridge/run_bridge_replay_case.sh"),
        "--suite",
        str(case.get("suite", "")),
        "--test-name",
        str(case.get("test_name", "")),
    ]
    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
    )

    values = parse_key_values(completed.stdout)
    status = values.get("status", "error")
    error = values.get("error", "")

    if completed.returncode == 2 and status == "error":
        return {"status": "error", "error": error or "runner configuration error"}

    if completed.returncode not in (0, 1):
        message = (completed.stderr.strip() or completed.stdout.strip() or "runner failed").replace(
            "\n", " "
        )
        return {"status": "error", "error": message}

    return {"status": status, "error": error}


def evaluate_case(case: Dict[str, Any]) -> Tuple[Dict[str, Any], bool]:
    expected = case.get("expected", {})
    expected_status = str(expected.get("status", "pass")) if isinstance(expected, dict) else "pass"

    result = run_case(case)
    actual_status = result.get("status", "error")

    mismatches: List[str] = []
    if actual_status != expected_status:
        mismatches.append(f"status expected {expected_status} got {actual_status}")

    passed = len(mismatches) == 0
    record: Dict[str, Any] = {
        "id": case.get("id", "unknown"),
        "suite": case.get("suite", ""),
        "test_name": case.get("test_name", ""),
        "class": case.get("class", ""),
        "expected_status": expected_status,
        "actual_status": actual_status,
        "error": result.get("error", ""),
        "passed": passed,
    }
    if mismatches:
        record["mismatches"] = mismatches
    return record, passed


def main() -> int:
    args = parse_args()
    fixture_path = Path(args.fixture)
    if not fixture_path.exists():
        print(f"status=fail; reason=fixture-not-found; fixture={fixture_path}")
        return 2

    fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
    cases = fixture.get("cases", [])
    if not isinstance(cases, list):
        print("status=fail; reason=invalid-fixture-cases")
        return 2

    requested_suites = parse_requested_suites(args.suites)
    if requested_suites:
        requested_set = set(requested_suites)
        selected_cases = [
            case
            for case in cases
            if isinstance(case, dict) and str(case.get("suite", "")) in requested_set
        ]
        if not selected_cases:
            print(
                "status=fail; "
                "reason=no-selected-cases; "
                f"requested_suites={','.join(requested_suites)}"
            )
            return 2
    else:
        selected_cases = cases

    report_cases: List[Dict[str, Any]] = []
    failed_ids: List[str] = []
    for case in selected_cases:
        if not isinstance(case, dict):
            failed_ids.append("invalid-case")
            continue
        record, passed = evaluate_case(case)
        report_cases.append(record)
        if not passed:
            failed_ids.append(str(case.get("id", "unknown")))

    status = "pass" if not failed_ids else "fail"
    report = {
        "status": status,
        "fixture": str(fixture_path),
        "case_count": len(selected_cases),
        "failed_count": len(failed_ids),
        "failed_case_ids": failed_ids,
        "requested_suites": requested_suites,
        "cases": report_cases,
    }

    if args.output_json:
        Path(args.output_json).write_text(json.dumps(report, indent=2), encoding="utf-8")

    if status == "pass":
        print(
            f"status=pass; cases={len(selected_cases)}; failed=0; "
            f"suites={','.join(requested_suites) if requested_suites else 'all'}"
        )
        return 0

    print(
        "status=fail; "
        f"cases={len(selected_cases)}; failed={len(failed_ids)}; "
        f"failed_ids={','.join(failed_ids)}"
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
