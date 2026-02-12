#!/usr/bin/env python3
"""Contract lane runner for Kolme-fork secp256k1 signature parity vectors."""

from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tempfile
import time
from pathlib import Path


ROOT_DIR = Path(__file__).resolve().parents[3]
MATRIX_RUNNER = ROOT_DIR / "scripts/kolme/run_signature_parity_matrix.py"
POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_signature_parity_policy.py"
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_commit/signature_parity_vectors.json"
ARCH_DOC = ROOT_DIR / "docs/architecture/kolme-live-integration.md"
CI_DOC = ROOT_DIR / "docs/ci/strategy.md"
MAX_SECONDS_ENV = "KAMN_KOLME_SIGNATURE_PARITY_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 120
REQUIRED_VECTOR_IDS = (
    "kolme_fork_primary_alpha",
    "kolme_fork_secondary_beta",
    "kolme_fork_primary_alpha_bad_signature",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run Kolme-fork signature parity contract lane checks."
    )
    parser.add_argument(
        "--output-json",
        default="/tmp/kolme-signature-parity-matrix-report.json",
        help="Signature parity matrix report output path.",
    )
    parser.add_argument(
        "--policy-output-json",
        default="/tmp/kolme-signature-parity-policy-report.json",
        help="Signature parity policy report output path.",
    )
    return parser.parse_args()


def parse_max_seconds() -> int:
    raw_value = os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip()
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError(f"{MAX_SECONDS_ENV} must be a positive integer")
    return int(raw_value)


def main() -> int:
    args = parse_args()

    if not os.access(MATRIX_RUNNER, os.X_OK):
        print("expected signature parity matrix runner to be executable", file=sys.stderr)
        return 1
    if not os.access(POLICY_CHECKER, os.X_OK):
        print("expected signature parity policy checker to be executable", file=sys.stderr)
        return 1
    if not FIXTURE_FILE.is_file():
        print("expected signature parity vector fixture to exist", file=sys.stderr)
        return 1
    if not ARCH_DOC.is_file():
        print("expected docs/architecture/kolme-live-integration.md to exist", file=sys.stderr)
        return 1
    if not CI_DOC.is_file():
        print("expected docs/ci/strategy.md to exist", file=sys.stderr)
        return 1

    try:
        max_seconds = parse_max_seconds()
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    start_epoch = time.monotonic()
    with tempfile.TemporaryDirectory(prefix="kolme-signature-parity-") as temp_dir:
        matrix_report = Path(args.output_json).resolve()
        policy_report = Path(args.policy_output_json).resolve()
        matrix_report.parent.mkdir(parents=True, exist_ok=True)
        policy_report.parent.mkdir(parents=True, exist_ok=True)

        matrix_run = subprocess.run(
            [
                "python3",
                str(MATRIX_RUNNER),
                "--fixture",
                str(FIXTURE_FILE),
                "--output-json",
                str(matrix_report),
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if matrix_run.returncode != 0:
            print(matrix_run.stdout, file=sys.stderr)
            print(matrix_run.stderr, file=sys.stderr)
            return matrix_run.returncode

        policy_run = subprocess.run(
            [
                "python3",
                str(POLICY_CHECKER),
                "--report-file",
                str(matrix_report),
                "--expected-final-decision",
                "GO",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(policy_report),
                "--require-vector-id",
                REQUIRED_VECTOR_IDS[0],
                "--require-vector-id",
                REQUIRED_VECTOR_IDS[1],
                "--require-vector-id",
                REQUIRED_VECTOR_IDS[2],
            ],
            cwd=ROOT_DIR,
            check=False,
            capture_output=True,
            text=True,
        )
        if policy_run.returncode != 0:
            print(policy_run.stdout, file=sys.stderr)
            print(policy_run.stderr, file=sys.stderr)
            return policy_run.returncode

        matrix_payload = json.loads(matrix_report.read_text(encoding="utf-8"))
        if matrix_payload.get("schema_version") != "kamn.kolme.signature-parity-matrix-report.v1":
            print("unexpected signature parity matrix report schema", file=sys.stderr)
            return 1
        if matrix_payload.get("status") != "pass":
            print("expected signature parity matrix status pass", file=sys.stderr)
            return 1
        if matrix_payload.get("source_contract") != "njfio/kolme_fork-secp256k1-v1":
            print("expected signature parity source contract marker", file=sys.stderr)
            return 1
        cases = matrix_payload.get("cases")
        if not isinstance(cases, list) or len(cases) < 3:
            print("expected at least three signature parity vectors in matrix report", file=sys.stderr)
            return 1
        bad_vector_cases = [
            case
            for case in cases
            if isinstance(case, dict)
            and case.get("vector_id") == "kolme_fork_primary_alpha_bad_signature"
        ]
        if len(bad_vector_cases) != 1:
            print("expected one known-bad signature parity vector case in matrix report", file=sys.stderr)
            return 1
        # Regression: #2299
        bad_vector_case = bad_vector_cases[0]
        if bad_vector_case.get("observed_final_decision") != "NO-GO":
            print("expected known-bad signature vector to observe NO-GO decision", file=sys.stderr)
            return 1
        reason_codes = bad_vector_case.get("reason_codes", [])
        if not isinstance(reason_codes, list) or "parity_signature_mismatch" not in reason_codes:
            print("expected known-bad signature vector to emit parity_signature_mismatch reason", file=sys.stderr)
            return 1

        policy_payload = json.loads(policy_report.read_text(encoding="utf-8"))
        if policy_payload.get("schema_version") != "kamn.kolme.signature-parity-policy-report.v1":
            print("unexpected signature parity policy report schema", file=sys.stderr)
            return 1
        if policy_payload.get("final_decision") != "GO":
            print("expected signature parity policy final_decision GO", file=sys.stderr)
            return 1
        if policy_payload.get("reason_codes") != []:
            print("expected no signature parity policy reason codes", file=sys.stderr)
            return 1

        architecture_doc_text = ARCH_DOC.read_text(encoding="utf-8")
        ci_doc_text = CI_DOC.read_text(encoding="utf-8")
        required_arch_markers = (
            "run_signature_parity_matrix.py",
            "check_signature_parity_policy.py",
            "run_signature_parity_contract_lane.sh",
            "fixtures/kolme_commit/signature_parity_vectors.json",
        )
        for marker in required_arch_markers:
            if marker not in architecture_doc_text:
                print(
                    f"expected architecture doc to include signature parity marker: {marker}",
                    file=sys.stderr,
                )
                return 1
        required_ci_markers = (
            "test_run_signature_parity_contract_lane.sh",
            "KAMN_KOLME_SIGNATURE_PARITY_MAX_SECONDS=120",
        )
        for marker in required_ci_markers:
            if marker not in ci_doc_text:
                print(
                    f"expected CI strategy doc to include signature parity marker: {marker}",
                    file=sys.stderr,
                )
                return 1

        elapsed_seconds = int(time.monotonic() - start_epoch)
        if elapsed_seconds > max_seconds:
            print(
                f"Kolme signature parity contract lane exceeded runtime budget: {elapsed_seconds}s",
                file=sys.stderr,
            )
            return 1

    print("Kolme signature parity contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
