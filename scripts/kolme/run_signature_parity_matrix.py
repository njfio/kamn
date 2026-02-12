#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
from pathlib import Path
from typing import Any


ROOT_DIR = Path(__file__).resolve().parents[2]
DEFAULT_FIXTURE = ROOT_DIR / "fixtures/kolme_commit/signature_parity_vectors.json"
VECTOR_TEST_NAME = "integration_kolme_live_signer_vector_probe_contract"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run Kolme-fork secp256k1 signature parity vectors through the KAMN adapter probe."
    )
    parser.add_argument("--fixture", default=str(DEFAULT_FIXTURE))
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-cases",
        type=int,
        default=0,
        help="Optional cap for first N vectors to evaluate.",
    )
    return parser.parse_args()


def parse_key_values(output: str) -> dict[str, str]:
    values: dict[str, str] = {}
    for line in output.splitlines():
        raw = line.strip()
        if "=" not in raw:
            continue
        key, value = raw.split("=", 1)
        values[key] = value
    return values


def list_value(value: Any) -> list[str]:
    if not isinstance(value, list):
        return []
    return [str(item) for item in value]


def classify_reason_codes(exit_code: int, output: str) -> list[str]:
    reason_codes: list[str] = []
    if "must match expected signature vector" in output:
        reason_codes.append("parity_signature_mismatch")
    if "must match expected recovery id vector" in output:
        reason_codes.append("parity_recovery_id_mismatch")
    if "must match expected pubkey vector" in output:
        reason_codes.append("parity_pubkey_mismatch")
    if exit_code != 0 and not reason_codes:
        reason_codes.append("parity_probe_failed")
    return reason_codes


def run_vector_case(vector: dict[str, Any]) -> subprocess.CompletedProcess[str]:
    env = os.environ.copy()
    env["KAMN_KOLME_SIGNATURE_VECTOR_PRIVATE_KEY_HEX"] = str(vector["private_key_hex"])
    env["KAMN_KOLME_SIGNATURE_VECTOR_MESSAGE"] = str(vector["message"])
    env["KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_SIGNATURE_HEX"] = str(
        vector["expected_signature_hex"]
    )
    env["KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_RECOVERY_ID"] = str(
        int(vector["expected_recovery_id"])
    )
    env["KAMN_KOLME_SIGNATURE_VECTOR_EXPECTED_PUBKEY_HEX"] = str(vector["expected_pubkey_hex"])

    command = [
        "cargo",
        "test",
        "-p",
        "kamn-node",
        VECTOR_TEST_NAME,
        "--",
        "--ignored",
        "--nocapture",
    ]
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        env=env,
        check=False,
        capture_output=True,
        text=True,
    )


def main() -> int:
    args = parse_args()
    fixture_path = Path(args.fixture).resolve()
    if not fixture_path.is_file():
        print(f"status=fail; reason=fixture-not-found; fixture={fixture_path}")
        return 2

    fixture = json.loads(fixture_path.read_text(encoding="utf-8"))
    if fixture.get("schema_version") != "kamn.kolme.signature-parity-vectors.v1":
        print("status=fail; reason=invalid-fixture-schema")
        return 2

    vectors = fixture.get("vectors")
    if not isinstance(vectors, list):
        print("status=fail; reason=invalid-vector-set")
        return 2
    selected_vectors = vectors[: args.max_cases] if args.max_cases > 0 else vectors

    failed_vector_ids: list[str] = []
    report_cases: list[dict[str, Any]] = []

    for index, vector in enumerate(selected_vectors):
        if not isinstance(vector, dict):
            vector_id = f"invalid-vector-{index}"
            failed_vector_ids.append(vector_id)
            report_cases.append(
                {
                    "vector_id": vector_id,
                    "passed": False,
                    "reason_codes": ["vector_payload_invalid"],
                    "error": "vector payload must be an object",
                }
            )
            continue

        required_fields = (
            "vector_id",
            "private_key_hex",
            "message",
            "expected_signature_hex",
            "expected_recovery_id",
            "expected_pubkey_hex",
        )
        missing_fields = [field for field in required_fields if field not in vector]
        vector_id = str(vector.get("vector_id", f"vector-{index}"))
        if missing_fields:
            failed_vector_ids.append(vector_id)
            report_cases.append(
                {
                    "vector_id": vector_id,
                    "passed": False,
                    "reason_codes": [f"missing_field:{field}" for field in missing_fields],
                    "error": "required vector fields missing",
                }
            )
            continue

        expected_final_decision = str(vector.get("expected_final_decision", "GO"))
        required_reason_codes = list_value(vector.get("required_reason_codes", []))
        if expected_final_decision not in ("GO", "NO-GO"):
            failed_vector_ids.append(vector_id)
            report_cases.append(
                {
                    "vector_id": vector_id,
                    "passed": False,
                    "reason_codes": ["expected_final_decision_invalid"],
                    "error": "expected_final_decision must be GO or NO-GO",
                }
            )
            continue

        result = run_vector_case(vector)
        combined_output = f"{result.stdout}\n{result.stderr}"
        observed_values = parse_key_values(combined_output)
        reason_codes = classify_reason_codes(result.returncode, combined_output)
        observed_final_decision = "GO" if not reason_codes else "NO-GO"
        missing_required_reason_codes = [
            reason for reason in required_reason_codes if reason not in reason_codes
        ]
        exit_matches_expected = (
            result.returncode == 0
            if expected_final_decision == "GO"
            else result.returncode != 0
        )

        passed = (
            observed_final_decision == expected_final_decision
            and exit_matches_expected
            and not missing_required_reason_codes
        )
        if not passed:
            failed_vector_ids.append(vector_id)

        report_cases.append(
            {
                "vector_id": vector_id,
                "expected_final_decision": expected_final_decision,
                "observed_final_decision": observed_final_decision,
                "required_reason_codes": required_reason_codes,
                "reason_codes": reason_codes,
                "missing_required_reason_codes": missing_required_reason_codes,
                "policy_exit_code": result.returncode,
                "observed_signature_hex": observed_values.get("signature_hex", ""),
                "observed_recovery_id": observed_values.get("recovery_id", ""),
                "observed_pubkey_hex": observed_values.get("pubkey_hex", ""),
                "passed": passed,
                "error": result.stderr.strip(),
            }
        )

    status = "pass" if not failed_vector_ids else "fail"
    report = {
        "schema_version": "kamn.kolme.signature-parity-matrix-report.v1",
        "status": status,
        "fixture": str(fixture_path),
        "source_contract": fixture.get("source_contract", ""),
        "vector_count": len(selected_vectors),
        "failed_count": len(failed_vector_ids),
        "failed_vector_ids": failed_vector_ids,
        "cases": report_cases,
    }

    if args.output_json:
        output_file = Path(args.output_json).resolve()
        output_file.parent.mkdir(parents=True, exist_ok=True)
        output_file.write_text(json.dumps(report, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    if status == "pass":
        print(f"status=pass; vectors={len(selected_vectors)}; failed=0")
        return 0

    print(
        "status=fail; "
        f"vectors={len(selected_vectors)}; failed={len(failed_vector_ids)}; "
        f"failed_ids={','.join(failed_vector_ids)}"
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
