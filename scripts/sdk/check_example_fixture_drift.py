#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
import json
import subprocess
import tempfile
from pathlib import Path
from typing import Any, Dict, List


ROOT_DIR = Path(__file__).resolve().parents[2]
DEFAULT_FIXTURE = ROOT_DIR / "fixtures/sdk_parity/register_validation_cases.json"
DEFAULT_SNAPSHOT = ROOT_DIR / "fixtures/sdk_parity/register_validation_snapshot.json"
MATRIX_RUNNER = ROOT_DIR / "scripts/sdk/run_sdk_parity_matrix.sh"

REPORT_SCHEMA = "kamn.sdk.example-fixture-drift-report.v1"
SNAPSHOT_SCHEMA = "kamn.sdk.example-fixture-snapshot.v1"
REASON_NONE = "none"
REASON_RUNNER_MISSING = "sdk_example_fixture_matrix_runner_missing"
REASON_MATRIX_FAILED = "sdk_example_fixture_matrix_failed"
REASON_MATRIX_REPORT_INVALID = "sdk_example_fixture_matrix_report_invalid"
REASON_SNAPSHOT_MISSING = "sdk_example_fixture_snapshot_missing"
REASON_SNAPSHOT_INVALID = "sdk_example_fixture_snapshot_invalid"
REASON_SNAPSHOT_SCHEMA_INVALID = "sdk_example_fixture_snapshot_schema_invalid"
REASON_SNAPSHOT_DRIFT = "sdk_example_fixture_snapshot_drift"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--fixture", default=str(DEFAULT_FIXTURE))
    parser.add_argument("--snapshot", default=str(DEFAULT_SNAPSHOT))
    parser.add_argument("--output-json", default="")
    parser.add_argument("--generated-snapshot-json", default="")
    return parser.parse_args()


def canonical_json(payload: Dict[str, Any]) -> str:
    return json.dumps(payload, sort_keys=True, separators=(",", ":"))


def to_rel_path(path: Path) -> str:
    try:
        return str(path.resolve().relative_to(ROOT_DIR))
    except ValueError:
        return str(path.resolve())


def normalize_case(case: Dict[str, Any]) -> Dict[str, Any]:
    results = case.get("results", {})
    rust = results.get("rust", {}) if isinstance(results, dict) else {}
    python = results.get("python", {}) if isinstance(results, dict) else {}
    typescript = results.get("typescript", {}) if isinstance(results, dict) else {}
    return {
        "id": str(case.get("id", "")),
        "expected_status": str(case.get("expected_status", "")),
        "expected_error_code": str(case.get("expected_error_code", "")),
        "rust": {
            "status": str(rust.get("status", "")),
            "error_code": str(rust.get("error_code", "")),
        },
        "python": {
            "status": str(python.get("status", "")),
            "error_code": str(python.get("error_code", "")),
        },
        "typescript": {
            "status": str(typescript.get("status", "")),
            "error_code": str(typescript.get("error_code", "")),
        },
        "passed": bool(case.get("passed", False)),
    }


def generate_snapshot_payload(matrix_report: Dict[str, Any], fixture_path: Path) -> Dict[str, Any]:
    cases = matrix_report.get("cases", [])
    if not isinstance(cases, list):
        raise ValueError("matrix report cases must be an array")

    normalized_cases = [normalize_case(case) for case in cases if isinstance(case, dict)]
    normalized_cases.sort(key=lambda case: case["id"])

    return {
        "schema_version": SNAPSHOT_SCHEMA,
        "source_matrix_schema": str(matrix_report.get("schema_version", "")),
        "fixture": to_rel_path(fixture_path),
        "case_count": len(normalized_cases),
        "cases": normalized_cases,
    }


def compute_drift_case_ids(expected_snapshot: Dict[str, Any], generated_snapshot: Dict[str, Any]) -> List[str]:
    expected_cases = expected_snapshot.get("cases", [])
    generated_cases = generated_snapshot.get("cases", [])
    expected_map = {
        str(case.get("id", "")): case for case in expected_cases if isinstance(case, dict)
    }
    generated_map = {
        str(case.get("id", "")): case for case in generated_cases if isinstance(case, dict)
    }

    drift_case_ids: List[str] = []
    for case_id in sorted(set(expected_map.keys()) | set(generated_map.keys())):
        if expected_map.get(case_id) != generated_map.get(case_id):
            drift_case_ids.append(case_id)
    return drift_case_ids


def load_json(path: Path) -> Dict[str, Any]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("json payload must be an object")
    return payload


def main() -> int:
    args = parse_args()
    fixture_path = Path(args.fixture).resolve()
    snapshot_path = Path(args.snapshot).resolve()

    reason_codes: List[str] = []
    generated_snapshot: Dict[str, Any] = {
        "schema_version": SNAPSHOT_SCHEMA,
        "source_matrix_schema": "",
        "fixture": to_rel_path(fixture_path),
        "case_count": 0,
        "cases": [],
    }
    expected_snapshot: Dict[str, Any] = {}
    drift_case_ids: List[str] = []

    if not MATRIX_RUNNER.exists() or not MATRIX_RUNNER.is_file():
        reason_codes.append(REASON_RUNNER_MISSING)
    elif not MATRIX_RUNNER.stat().st_mode & 0o111:
        reason_codes.append(REASON_RUNNER_MISSING)
    else:
        with tempfile.TemporaryDirectory() as tmp_dir:
            matrix_report_path = Path(tmp_dir) / "sdk-parity-matrix-report.json"
            completed = subprocess.run(
                [
                    "bash",
                    str(MATRIX_RUNNER),
                    "--fixture",
                    str(fixture_path),
                    "--output-json",
                    str(matrix_report_path),
                ],
                cwd=ROOT_DIR,
                text=True,
                capture_output=True,
            )

            if completed.returncode != 0:
                reason_codes.append(REASON_MATRIX_FAILED)
            else:
                try:
                    matrix_report = load_json(matrix_report_path)
                    generated_snapshot = generate_snapshot_payload(matrix_report, fixture_path)
                except (ValueError, json.JSONDecodeError):
                    reason_codes.append(REASON_MATRIX_REPORT_INVALID)

    if not snapshot_path.exists():
        reason_codes.append(REASON_SNAPSHOT_MISSING)
    else:
        try:
            expected_snapshot = load_json(snapshot_path)
        except (ValueError, json.JSONDecodeError):
            reason_codes.append(REASON_SNAPSHOT_INVALID)
        else:
            if expected_snapshot.get("schema_version") != SNAPSHOT_SCHEMA:
                reason_codes.append(REASON_SNAPSHOT_SCHEMA_INVALID)
            elif canonical_json(expected_snapshot) != canonical_json(generated_snapshot):
                reason_codes.append(REASON_SNAPSHOT_DRIFT)
                drift_case_ids = compute_drift_case_ids(expected_snapshot, generated_snapshot)

    if args.generated_snapshot_json:
        Path(args.generated_snapshot_json).write_text(
            json.dumps(generated_snapshot, indent=2) + "\n",
            encoding="utf-8",
        )

    reason_codes = sorted(set(reason_codes))
    status = "pass" if not reason_codes else "fail"
    if status == "pass":
        reason_codes = [REASON_NONE]

    generated_hash = hashlib.sha256(canonical_json(generated_snapshot).encode("utf-8")).hexdigest()
    expected_hash = ""
    if expected_snapshot:
        expected_hash = hashlib.sha256(canonical_json(expected_snapshot).encode("utf-8")).hexdigest()

    report = {
        "schema_version": REPORT_SCHEMA,
        "status": status,
        "reason_codes": reason_codes,
        "fixture": to_rel_path(fixture_path),
        "snapshot": to_rel_path(snapshot_path),
        "case_count": int(generated_snapshot.get("case_count", 0)),
        "drift_case_ids": drift_case_ids,
        "generated_snapshot_sha256": generated_hash,
        "expected_snapshot_sha256": expected_hash,
    }

    if args.output_json:
        Path(args.output_json).write_text(
            json.dumps(report, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"schema_version={REPORT_SCHEMA}")
    print(f"reason_codes={','.join(reason_codes)}")
    print(f"case_count={report['case_count']}")
    if drift_case_ids:
        print(f"drift_case_ids={','.join(drift_case_ids)}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
