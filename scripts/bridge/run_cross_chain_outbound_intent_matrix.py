#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Any, Dict, List


ROOT_DIR = Path(__file__).resolve().parents[2]
GENERATOR = ROOT_DIR / "scripts/bridge/generate_cross_chain_outbound_intent_evidence_bundle.sh"
POLICY_CHECKER = ROOT_DIR / "scripts/bridge/check_cross_chain_outbound_intent_policy.sh"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fixture",
        default=str(ROOT_DIR / "fixtures/bridge_outbound_intent/approval_retry_cases.json"),
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


def run_generator(case: Dict[str, Any], bundle_path: Path) -> subprocess.CompletedProcess[str]:
    command = [
        "bash",
        str(GENERATOR),
        "--output-file",
        str(bundle_path),
        "--chain",
        str(case["chain"]),
        "--request-id",
        str(case["request_id"]),
        "--destination-channel",
        str(case["destination_channel"]),
        "--required-approvals",
        str(case["required_approvals"]),
        "--received-approvals",
        str(case["received_approvals"]),
        "--approval-quorum-hash",
        str(case["approval_quorum_hash"]),
        "--idempotency-key",
        str(case["idempotency_key"]),
        "--attempt-number",
        str(case["attempt_number"]),
        "--payload-hash",
        str(case["payload_hash"]),
        "--previous-payload-hash",
        str(case["previous_payload_hash"]),
        "--duplicate-request",
        "true" if case["duplicate_request"] else "false",
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
    cases = fixture.get("cases")
    if not isinstance(cases, list):
        print("status=fail; reason=invalid-fixture-cases")
        return 2

    selected_cases = cases[: args.max_cases] if args.max_cases > 0 else cases
    tmp_dir = Path(
        subprocess.check_output(
            ["mktemp", "-d"], text=True, cwd=ROOT_DIR
        ).strip()
    )
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
            expected = str(case.get("expected_final_decision", "GO"))

            generated = run_generator(case, bundle_path)
            if generated.returncode != 0:
                failed_case_ids.append(case_id)
                report_cases.append(
                    {
                        "case_id": case_id,
                        "expected_final_decision": expected,
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
                        "expected_final_decision": expected,
                        "generated_final_decision": generated_decision,
                        "passed": False,
                        "stage": "policy-check",
                        "error": (checked.stderr.strip() or checked.stdout.strip()),
                    }
                )
                continue

            checked_values = parse_key_values(checked.stdout)
            checked_decision = checked_values.get("final_decision", "")

            passed = generated_decision == expected and checked_decision == expected
            if not passed:
                failed_case_ids.append(case_id)

            report_cases.append(
                {
                    "case_id": case_id,
                    "expected_final_decision": expected,
                    "generated_final_decision": generated_decision,
                    "checked_final_decision": checked_decision,
                    "passed": passed,
                }
            )

        status = "pass" if not failed_case_ids else "fail"
        report = {
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
