#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parents[2]
POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_nonce_broadcast_parity_policy.py"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run deterministic nonce/broadcast parity policy matrix."
    )
    parser.add_argument(
        "--fixture",
        default=str(ROOT_DIR / "fixtures/kolme_commit/nonce_broadcast_parity_cases.json"),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-cases",
        type=int,
        default=0,
        help="Optional cap for first N cases to evaluate.",
    )
    return parser.parse_args()


def parse_key_values(stdout: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in stdout.splitlines():
        raw = line.strip()
        if not raw or "=" not in raw:
            continue
        key, value = raw.split("=", 1)
        values[key] = value
    return values


def bool_value(value: Any) -> str:
    return "true" if bool(value) else "false"


def list_value(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value]


def run_policy_checker(case: dict[str, Any], report_path: Path) -> subprocess.CompletedProcess[str]:
    command = [
        "python3",
        str(POLICY_CHECKER),
        "--case-id",
        str(case["case_id"]),
        "--operation",
        str(case["operation"]),
        "--http-status",
        str(case["http_status"]),
        "--nonce-value",
        str(case.get("nonce_value", -1)),
        "--broadcast-accepted",
        bool_value(case.get("broadcast_accepted", False)),
        "--duplicate-detected",
        bool_value(case.get("duplicate_detected", False)),
        "--payload-valid",
        bool_value(case.get("payload_valid", True)),
        "--authorization-present",
        bool_value(case.get("authorization_present", True)),
        "--ci-fast-gate",
        str(case["ci_fast_gate"]),
        "--output-json",
        str(report_path),
    ]
    return subprocess.run(command, cwd=ROOT_DIR, text=True, capture_output=True)


def main() -> int:
    args = parse_args()
    fixture_path = Path(args.fixture)
    if not fixture_path.exists():
        print(f"status=fail; reason=fixture-not-found; fixture={fixture_path}")
        return 2
    if not POLICY_CHECKER.exists():
        print(f"status=fail; reason=policy-checker-not-found; path={POLICY_CHECKER}")
        return 2

    fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
    if fixture.get("schema_version") != "kamn.kolme.nonce-broadcast-parity-cases.v1":
        print("status=fail; reason=invalid-fixture-schema")
        return 2
    cases = fixture.get("cases")
    if not isinstance(cases, list):
        print("status=fail; reason=invalid-fixture-cases")
        return 2

    selected_cases = cases[: args.max_cases] if args.max_cases > 0 else cases
    tmp_dir = Path(subprocess.check_output(["mktemp", "-d"], text=True, cwd=ROOT_DIR).strip())

    try:
        failed_case_ids: list[str] = []
        report_cases: list[dict[str, Any]] = []

        for index, case in enumerate(selected_cases):
            if not isinstance(case, dict):
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

            case_id = str(case.get("case_id", f"case-{index}"))
            expected = str(case.get("expected_final_decision", "GO"))
            required_reason_codes = list_value(case.get("required_reason_codes", []))
            checker_report_path = tmp_dir / f"{case_id}.json"

            checked = run_policy_checker(case, checker_report_path)
            values = parse_key_values(checked.stdout)
            observed = values.get("final_decision", "")
            failed_checks_raw = values.get("failed_checks", "")
            failed_checks = [] if failed_checks_raw in ("", "none") else failed_checks_raw.split(",")
            expected_exit = 0 if expected == "GO" else 1

            missing_required = [
                reason for reason in required_reason_codes if reason not in failed_checks
            ]
            passed = (
                observed == expected
                and checked.returncode == expected_exit
                and not missing_required
            )
            if not passed:
                failed_case_ids.append(case_id)

            report_cases.append(
                {
                    "case_id": case_id,
                    "expected_final_decision": expected,
                    "observed_final_decision": observed,
                    "required_reason_codes": required_reason_codes,
                    "failed_checks": failed_checks,
                    "missing_required_reason_codes": missing_required,
                    "policy_exit_code": checked.returncode,
                    "passed": passed,
                    "error": checked.stderr.strip(),
                }
            )

        status = "pass" if not failed_case_ids else "fail"
        report = {
            "schema_version": "kamn.kolme.nonce-broadcast-parity-matrix-report.v1",
            "status": status,
            "fixture": str(fixture_path),
            "case_count": len(selected_cases),
            "failed_count": len(failed_case_ids),
            "failed_case_ids": failed_case_ids,
            "cases": report_cases,
        }

        if args.output_json:
            Path(args.output_json).write_text(
                json.dumps(report, indent=2, sort_keys=True) + "\n",
                encoding="utf-8",
            )

        if status == "pass":
            print(f"status=pass; cases={len(selected_cases)}; failed=0")
            return 0

        print(
            "status=fail; "
            f"cases={len(selected_cases)}; failed={len(failed_case_ids)}; "
            f"failed_ids={','.join(failed_case_ids)}"
        )
        return 1
    finally:
        subprocess.run(["rm", "-rf", str(tmp_dir)], check=False)


if __name__ == "__main__":
    raise SystemExit(main())
