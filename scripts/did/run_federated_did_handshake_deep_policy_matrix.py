#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parents[2]
CHECKER = ROOT_DIR / "scripts/did/check_federated_did_handshake_deep_policy.sh"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fixture",
        default=str(ROOT_DIR / "fixtures/federated_did_handshake/deep_lane_policy_cases.json"),
    )
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def parse_key_values(stdout: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in stdout.splitlines():
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def run_checker(report_file: Path) -> subprocess.CompletedProcess[str]:
    command = [
        "bash",
        str(CHECKER),
        "--report-file",
        str(report_file),
    ]
    return subprocess.run(command, cwd=ROOT_DIR, text=True, capture_output=True)


def main() -> int:
    args = parse_args()
    fixture_path = Path(args.fixture)
    if not fixture_path.is_file():
        print(f"status=fail; reason=fixture-not-found; fixture={fixture_path}")
        return 2

    if not CHECKER.is_file():
        print(f"status=fail; reason=checker-missing; checker={CHECKER}")
        return 2

    payload = json.loads(fixture_path.read_text(encoding="utf-8"))
    if payload.get("schema_version") != "kamn.did.federated-handshake.deep-policy-cases.v1":
        print("status=fail; reason=invalid-fixture-schema")
        return 2

    cases = payload.get("cases")
    if not isinstance(cases, list) or not cases:
        print("status=fail; reason=invalid-fixture-cases")
        return 2

    tmp_dir = Path(subprocess.check_output(["mktemp", "-d"], text=True, cwd=ROOT_DIR).strip())
    report_cases: list[dict[str, Any]] = []
    failed_case_ids: list[str] = []
    try:
        for index, case in enumerate(cases):
            if not isinstance(case, dict):
                case_id = f"invalid-case-{index}"
                failed_case_ids.append(case_id)
                report_cases.append(
                    {
                        "case_id": case_id,
                        "passed": False,
                        "error": "case entry must be an object",
                    }
                )
                continue

            case_id = str(case.get("case_id", f"case-{index}"))
            expected_policy_status = str(case.get("expected_policy_status", "pass"))
            expected_final_decision = str(case.get("expected_final_decision", "GO"))
            report_payload = case.get("report")
            if not isinstance(report_payload, dict):
                failed_case_ids.append(case_id)
                report_cases.append(
                    {
                        "case_id": case_id,
                        "expected_policy_status": expected_policy_status,
                        "expected_final_decision": expected_final_decision,
                        "passed": False,
                        "error": "report payload must be an object",
                    }
                )
                continue

            report_file = tmp_dir / f"{case_id}.json"
            report_file.write_text(
                json.dumps(report_payload, sort_keys=True, indent=2) + "\n",
                encoding="utf-8",
            )

            checked = run_checker(report_file)
            actual_policy_status = "pass" if checked.returncode == 0 else "fail"
            checked_values = parse_key_values(checked.stdout)
            actual_final_decision = checked_values.get("final_decision", "")

            passed = actual_policy_status == expected_policy_status
            if expected_policy_status == "pass":
                passed = passed and actual_final_decision == expected_final_decision

            if not passed:
                failed_case_ids.append(case_id)

            report_cases.append(
                {
                    "case_id": case_id,
                    "expected_policy_status": expected_policy_status,
                    "actual_policy_status": actual_policy_status,
                    "expected_final_decision": expected_final_decision,
                    "actual_final_decision": actual_final_decision,
                    "passed": passed,
                    "checker_stdout": checked.stdout.strip(),
                    "checker_stderr": checked.stderr.strip(),
                }
            )

        status = "pass" if not failed_case_ids else "fail"
        report = {
            "schema_version": "kamn.did.federated-handshake.deep-policy-matrix.v1",
            "status": status,
            "fixture": str(fixture_path),
            "case_count": len(cases),
            "failed_count": len(failed_case_ids),
            "failed_case_ids": failed_case_ids,
            "cases": report_cases,
        }

        if args.output_json:
            Path(args.output_json).write_text(
                json.dumps(report, sort_keys=True, indent=2) + "\n",
                encoding="utf-8",
            )

        if status == "pass":
            print(f"status=pass; cases={len(cases)}; failed=0")
            return 0

        print(
            "status=fail; "
            f"cases={len(cases)}; failed={len(failed_case_ids)}; "
            f"failed_ids={','.join(failed_case_ids)}"
        )
        return 1
    finally:
        subprocess.run(["rm", "-rf", str(tmp_dir)], check=False)


if __name__ == "__main__":
    raise SystemExit(main())
