#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import subprocess
from pathlib import Path
from typing import Dict, List, Tuple


ROOT_DIR = Path(__file__).resolve().parents[2]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--fixture",
        default=str(ROOT_DIR / "fixtures/sdk_parity/register_validation_cases.json"),
    )
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def normalize_error_code(status: str, error: str) -> str:
    if status != "error":
        return ""

    lower = error.lower()
    if ("agent_type" in lower or "agenttype" in lower) and "empty" in lower:
        return "agent_type_empty"
    if "model_family" in lower and "empty" in lower:
        return "model_family_empty"
    if "capabilities" in lower and "must not be empty" in lower:
        return "capabilities_empty"
    if "at least one capability" in lower:
        return "capabilities_empty"
    if "empty capability" in lower:
        return "capabilities_empty_entry"
    if "capabilities must not include empty entries" in lower:
        return "capabilities_empty_entry"
    return "unknown_error"


def parse_key_values(stdout: str) -> Dict[str, str]:
    result: Dict[str, str] = {}
    for raw_line in stdout.splitlines():
        line = raw_line.strip()
        if not line or "=" not in line:
            continue
        key, value = line.split("=", 1)
        result[key] = value
    return result


def run_runner(script: str, case: Dict[str, object]) -> Dict[str, str]:
    command: List[str] = [
        "bash",
        str(ROOT_DIR / script),
        "--agent-type",
        str(case.get("agent_type", "")),
        "--model-family",
        str(case.get("model_family", "")),
    ]
    capabilities = case.get("capabilities", [])
    if isinstance(capabilities, list):
        for capability in capabilities:
            command.extend(["--capability", str(capability)])

    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        text=True,
        capture_output=True,
    )

    if completed.returncode != 0:
        message = (completed.stderr.strip() or completed.stdout.strip() or "runner failed").replace(
            "\n", " "
        )
        return {
            "status": "error",
            "error": message,
            "error_code": "runner_failed",
        }

    values = parse_key_values(completed.stdout)
    status = values.get("status", "error")
    error = values.get("error", "")
    error_code = normalize_error_code(status, error)
    return {"status": status, "error": error, "error_code": error_code}


def evaluate_case(case: Dict[str, object]) -> Tuple[Dict[str, object], bool]:
    expected = case.get("expected", {})
    expected_status = str(expected.get("status", "error")) if isinstance(expected, dict) else "error"
    expected_error_code = (
        str(expected.get("error_code", ""))
        if isinstance(expected, dict) and expected_status == "error"
        else ""
    )

    runner_map = {
        "rust": run_runner("scripts/sdk/run_parity_rust.sh", case),
        "python": run_runner("scripts/sdk/run_parity_python.sh", case),
        "typescript": run_runner("scripts/sdk/run_parity_typescript.sh", case),
    }

    mismatch_reasons: List[str] = []

    for runner_name, result in runner_map.items():
        if result["status"] != expected_status:
            mismatch_reasons.append(
                f"{runner_name}: status expected {expected_status} got {result['status']}"
            )
        if expected_status == "error" and expected_error_code:
            if result["error_code"] != expected_error_code:
                mismatch_reasons.append(
                    f"{runner_name}: error_code expected {expected_error_code} got {result['error_code']}"
                )

    parity_signature = {(v["status"], v["error_code"]) for v in runner_map.values()}
    if len(parity_signature) != 1:
        mismatch_reasons.append("cross-language parity mismatch")

    passed = len(mismatch_reasons) == 0
    record: Dict[str, object] = {
        "id": case.get("id", "unknown"),
        "expected_status": expected_status,
        "expected_error_code": expected_error_code,
        "results": runner_map,
        "passed": passed,
    }
    if mismatch_reasons:
        record["mismatches"] = mismatch_reasons
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

    report_cases: List[Dict[str, object]] = []
    failed_ids: List[str] = []
    for case in cases:
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
        "case_count": len(cases),
        "failed_count": len(failed_ids),
        "failed_case_ids": failed_ids,
        "cases": report_cases,
    }

    if args.output_json:
        Path(args.output_json).write_text(json.dumps(report, indent=2), encoding="utf-8")

    if status == "pass":
        print(f"status=pass; cases={len(cases)}; failed=0")
        return 0

    print(
        "status=fail; "
        f"cases={len(cases)}; failed={len(failed_ids)}; "
        f"failed_ids={','.join(failed_ids)}"
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
