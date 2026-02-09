#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
import tempfile
from pathlib import Path
from typing import Any, Dict


ROOT_DIR = Path(__file__).resolve().parents[2]
VALIDATOR = ROOT_DIR / "scripts/kolme/validate_version_compatibility.py"


def _parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run deterministic Kolme version compatibility replay matrix."
    )
    parser.add_argument(
        "--fixture",
        default=str(ROOT_DIR / "fixtures/kolme_compatibility/version_compatibility_cases.json"),
    )
    parser.add_argument("--output-json", required=True)
    parser.add_argument("--max-cases", type=int, default=0)
    return parser.parse_args()


def _parse_kv(stdout: str) -> Dict[str, str]:
    values: Dict[str, str] = {}
    for line in stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def main() -> int:
    args = _parse_args()
    fixture_file = Path(args.fixture).resolve()
    output_file = Path(args.output_json).resolve()
    output_file.parent.mkdir(parents=True, exist_ok=True)

    fixture = json.loads(fixture_file.read_text(encoding="utf-8"))
    if fixture.get("schema_version") != "kamn.kolme.version-compatibility-cases.v1":
        raise SystemExit("unexpected version compatibility fixture schema")
    cases = fixture.get("cases")
    if not isinstance(cases, list) or not cases:
        raise SystemExit("version compatibility fixture must contain non-empty cases")

    selected_cases = cases[: args.max_cases] if args.max_cases > 0 else cases
    report_cases: list[dict[str, Any]] = []
    failed_case_ids: list[str] = []

    with tempfile.TemporaryDirectory() as tmp_dir:
        tmp_root = Path(tmp_dir)
        for index, raw_case in enumerate(selected_cases):
            if not isinstance(raw_case, dict):
                case_id = f"invalid-case-{index}"
                failed_case_ids.append(case_id)
                report_cases.append(
                    {
                        "case_id": case_id,
                        "passed": False,
                        "error": "case payload is not an object",
                    }
                )
                continue

            case_id = str(raw_case.get("case_id", f"case-{index}"))
            expected_decision = str(raw_case.get("expected_final_decision", "GO"))
            expected_reason_codes = raw_case.get("expected_reason_codes", [])
            if not isinstance(expected_reason_codes, list):
                raise SystemExit(f"{case_id}: expected_reason_codes must be an array")

            case_report = tmp_root / f"{case_id}.json"
            command = [
                "python3",
                str(VALIDATOR),
                "--kamn-version",
                str(raw_case["kamn_version"]),
                "--kolme-release-tag",
                str(raw_case["kolme_release_tag"]),
                "--ci-fast-gate",
                str(raw_case["ci_fast_gate"]),
                "--output-json",
                str(case_report),
            ]
            completed = subprocess.run(command, cwd=ROOT_DIR, text=True, capture_output=True)
            values = _parse_kv(completed.stdout + "\n" + completed.stderr)
            actual_decision = values.get("final_decision", "")
            failed_checks = values.get("failed_checks", "")
            actual_reason_codes = [] if failed_checks in {"", "none"} else failed_checks.split(",")

            expected_go = expected_decision == "GO"
            if expected_go and completed.returncode != 0:
                failed_case_ids.append(case_id)
            if not expected_go and completed.returncode == 0:
                failed_case_ids.append(case_id)

            reasons_match = all(reason in actual_reason_codes for reason in expected_reason_codes)
            decision_match = actual_decision == expected_decision
            passed = decision_match and reasons_match and (
                (completed.returncode == 0 and expected_go)
                or (completed.returncode != 0 and not expected_go)
            )
            if not passed and case_id not in failed_case_ids:
                failed_case_ids.append(case_id)

            report_cases.append(
                {
                    "case_id": case_id,
                    "expected_final_decision": expected_decision,
                    "actual_final_decision": actual_decision,
                    "expected_reason_codes": expected_reason_codes,
                    "actual_reason_codes": actual_reason_codes,
                    "passed": passed,
                }
            )

    status = "pass" if not failed_case_ids else "fail"
    report = {
        "schema_version": "kamn.kolme.version-compatibility-replay-report.v1",
        "fixture": str(fixture_file),
        "status": status,
        "case_count": len(selected_cases),
        "failed_count": len(failed_case_ids),
        "failed_case_ids": failed_case_ids,
        "cases": report_cases,
    }
    output_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    if status == "pass":
        print(f"status=pass; cases={len(selected_cases)}; failed=0")
        return 0

    print(
        "status=fail; "
        f"cases={len(selected_cases)}; failed={len(failed_case_ids)}; "
        f"failed_ids={','.join(failed_case_ids)}"
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
