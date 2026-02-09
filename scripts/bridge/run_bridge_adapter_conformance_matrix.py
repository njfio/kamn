#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any, Dict, List


ROOT_DIR = Path(__file__).resolve().parents[2]
GENERATOR = ROOT_DIR / "scripts/bridge/generate_bridge_adapter_conformance_evidence_bundle.sh"
POLICY_CHECKER = ROOT_DIR / "scripts/bridge/check_bridge_adapter_conformance_policy.sh"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fixture",
        default=str(
            ROOT_DIR
            / "fixtures/bridge_adapter_conformance/request_receipt_schema_cases.json"
        ),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-cases",
        type=int,
        default=0,
        help="Optional cap for first N cases to evaluate.",
    )
    return parser.parse_args()


def parse_key_values(stdout: str) -> Dict[str, str]:
    values: Dict[str, str] = {}
    for line in stdout.splitlines():
        raw = line.strip()
        if not raw or "=" not in raw:
            continue
        key, value = raw.split("=", 1)
        values[key] = value
    return values


def csv_fields(values: Any) -> str:
    if isinstance(values, list):
        normalized = sorted({str(item).strip() for item in values if str(item).strip()})
        return ",".join(normalized)
    return ""


def run_generator(case: Dict[str, Any], bundle_path: Path) -> subprocess.CompletedProcess[str]:
    command = [
        "bash",
        str(GENERATOR),
        "--output-file",
        str(bundle_path),
        "--adapter-id",
        str(case["adapter_id"]),
        "--bridge-network",
        str(case["bridge_network"]),
        "--dry-run",
        "true" if bool(case["dry_run"]) else "false",
        "--request-expected-schema-version",
        str(case["request_expected_schema_version"]),
        "--request-observed-schema-version",
        str(case["request_observed_schema_version"]),
        "--request-required-fields",
        csv_fields(case["request_required_fields"]),
        "--request-observed-fields",
        csv_fields(case["request_observed_fields"]),
        "--receipt-expected-schema-version",
        str(case["receipt_expected_schema_version"]),
        "--receipt-observed-schema-version",
        str(case["receipt_observed_schema_version"]),
        "--receipt-required-fields",
        csv_fields(case["receipt_required_fields"]),
        "--receipt-observed-fields",
        csv_fields(case["receipt_observed_fields"]),
        "--ci-fast-gate",
        str(case["ci_fast_gate"]),
    ]
    return subprocess.run(command, cwd=ROOT_DIR, text=True, capture_output=True)


def run_policy_checker(bundle_path: Path) -> subprocess.CompletedProcess[str]:
    command = [
        "bash",
        str(POLICY_CHECKER),
        "--bundle-file",
        str(bundle_path),
    ]
    return subprocess.run(command, cwd=ROOT_DIR, text=True, capture_output=True)


def main() -> int:
    args = parse_args()
    fixture_path = Path(args.fixture)
    if not fixture_path.exists():
        print(f"status=fail; reason=fixture-not-found; fixture={fixture_path}")
        return 2

    fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
    if fixture.get("schema_version") != "kamn.bridge.adapter-conformance.fixture.v1":
        print("status=fail; reason=invalid-fixture-schema")
        return 2

    cases = fixture.get("cases")
    if not isinstance(cases, list):
        print("status=fail; reason=invalid-fixture-cases")
        return 2

    selected_cases = cases[: args.max_cases] if args.max_cases > 0 else cases
    tmp_dir = Path(subprocess.check_output(["mktemp", "-d"], text=True, cwd=ROOT_DIR).strip())
    try:
        failed_case_ids: List[str] = []
        report_cases: List[Dict[str, Any]] = []

        for index, case in enumerate(selected_cases):
            if not isinstance(case, dict):
                failed_case_ids.append(f"invalid-case-{index}")
                report_cases.append(
                    {
                        "case_id": f"invalid-case-{index}",
                        "passed": False,
                        "error": "case payload is not an object",
                    }
                )
                continue

            case_id = str(case.get("case_id", f"case-{index}"))
            bundle_path = tmp_dir / f"{case_id}.json"
            expected_decision = str(case.get("expected_final_decision", "GO"))

            generated = run_generator(case, bundle_path)
            if generated.returncode != 0:
                failed_case_ids.append(case_id)
                report_cases.append(
                    {
                        "case_id": case_id,
                        "expected_final_decision": expected_decision,
                        "passed": False,
                        "stage": "generate",
                        "error": (generated.stderr.strip() or generated.stdout.strip()),
                    }
                )
                continue

            generated_values = parse_key_values(generated.stdout)
            generated_decision = generated_values.get("final_decision", "")

            checked = run_policy_checker(bundle_path)
            if checked.returncode != 0:
                failed_case_ids.append(case_id)
                report_cases.append(
                    {
                        "case_id": case_id,
                        "expected_final_decision": expected_decision,
                        "generated_final_decision": generated_decision,
                        "passed": False,
                        "stage": "policy-check",
                        "error": (checked.stderr.strip() or checked.stdout.strip()),
                    }
                )
                continue

            checked_values = parse_key_values(checked.stdout)
            checked_decision = checked_values.get("final_decision", "")

            passed = generated_decision == expected_decision and checked_decision == expected_decision
            if not passed:
                failed_case_ids.append(case_id)

            report_cases.append(
                {
                    "case_id": case_id,
                    "expected_final_decision": expected_decision,
                    "generated_final_decision": generated_decision,
                    "checked_final_decision": checked_decision,
                    "passed": passed,
                }
            )

        status = "pass" if not failed_case_ids else "fail"
        report = {
            "schema_version": "kamn.bridge.adapter-conformance.matrix-report.v1",
            "status": status,
            "fixture": str(fixture_path),
            "case_count": len(selected_cases),
            "failed_count": len(failed_case_ids),
            "failed_case_ids": failed_case_ids,
            "cases": report_cases,
        }

        if args.output_json:
            Path(args.output_json).write_text(
                json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
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
