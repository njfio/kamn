#!/usr/bin/env python3
"""Contract lane runner for Kolme nonce/broadcast parity checks."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_nonce_broadcast_parity_policy.py"
MATRIX_RUNNER = ROOT_DIR / "scripts/kolme/run_nonce_broadcast_parity_matrix.py"
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_commit/nonce_broadcast_parity_cases.json"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
CI_STRATEGY_DOC = ROOT_DIR / "docs/ci/strategy.md"
MAX_SECONDS_ENV = "KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS"
DEFAULT_MAX_SECONDS = 60


def parse_max_seconds() -> int:
    raw_value = os.environ.get(MAX_SECONDS_ENV, str(DEFAULT_MAX_SECONDS)).strip()
    if not raw_value.isdigit() or int(raw_value) <= 0:
        raise ValueError(f"{MAX_SECONDS_ENV} must be a positive integer")
    return int(raw_value)


def command_output(command: list[str]) -> tuple[int, str]:
    result = subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def main() -> int:
    if not os.access(POLICY_CHECKER, os.X_OK):
        print("expected nonce/broadcast parity policy checker to be executable", file=sys.stderr)
        return 1
    if not os.access(MATRIX_RUNNER, os.X_OK):
        print("expected nonce/broadcast parity matrix runner to be executable", file=sys.stderr)
        return 1
    if not FIXTURE_FILE.is_file():
        print("expected nonce/broadcast parity fixture file to exist", file=sys.stderr)
        return 1
    if not ROADMAP_DOC.is_file() or not CI_STRATEGY_DOC.is_file():
        print("expected Kolme roadmap and CI strategy docs to exist", file=sys.stderr)
        return 1

    try:
        max_seconds = parse_max_seconds()
    except ValueError as error:
        print(str(error), file=sys.stderr)
        return 1

    start_epoch = time.monotonic()
    tmp_report = Path(subprocess.check_output(["mktemp"], text=True).strip())
    try:
        go_code, go_output = command_output(
            [
                "python3",
                str(POLICY_CHECKER),
                "--case-id",
                "broadcast-duplicate-go-lane-001",
                "--operation",
                "broadcast",
                "--http-status",
                "409",
                "--nonce-value",
                "0",
                "--broadcast-accepted",
                "false",
                "--duplicate-detected",
                "true",
                "--payload-valid",
                "true",
                "--authorization-present",
                "true",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(tmp_report),
            ]
        )
        if go_code != 0:
            print(go_output, file=sys.stderr)
            return go_code
        if "final_decision=GO" not in go_output:
            print("expected duplicate broadcast parity case to produce GO", file=sys.stderr)
            return 1

        no_go_code, no_go_output = command_output(
            [
                "python3",
                str(POLICY_CHECKER),
                "--case-id",
                "broadcast-unauthorized-no-go-lane-001",
                "--operation",
                "broadcast",
                "--http-status",
                "401",
                "--nonce-value",
                "0",
                "--broadcast-accepted",
                "false",
                "--duplicate-detected",
                "false",
                "--payload-valid",
                "true",
                "--authorization-present",
                "false",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(tmp_report),
            ]
        )
        if no_go_code == 0:
            print("expected unauthorized broadcast parity case to fail closed", file=sys.stderr)
            return 1
        if "final_decision=NO-GO" not in no_go_output:
            print("expected unauthorized broadcast parity case to produce NO-GO", file=sys.stderr)
            return 1
        if "unauthorized_status" not in no_go_output:
            print(
                "expected unauthorized broadcast parity case to emit unauthorized_status reason code",
                file=sys.stderr,
            )
            return 1

        matrix_code, matrix_output = command_output(
            [
                "python3",
                str(MATRIX_RUNNER),
                "--fixture",
                str(FIXTURE_FILE),
                "--max-cases",
                "5",
                "--output-json",
                str(tmp_report),
            ]
        )
        if matrix_code != 0:
            print(matrix_output, file=sys.stderr)
            return matrix_code
        if "status=pass;" not in matrix_output:
            print("expected nonce/broadcast parity matrix to pass for fixture cases", file=sys.stderr)
            return 1

        matrix_payload = json.loads(tmp_report.read_text(encoding="utf-8"))
        if (
            matrix_payload.get("schema_version")
            != "kamn.kolme.nonce-broadcast-parity-matrix-report.v1"
        ):
            print("unexpected nonce/broadcast parity matrix report schema", file=sys.stderr)
            return 1
        if matrix_payload.get("status") != "pass":
            print("expected nonce/broadcast parity matrix report to pass", file=sys.stderr)
            return 1
        cases = matrix_payload.get("cases", [])
        if not any(case.get("case_id") == "broadcast_duplicate_idempotent_go" for case in cases):
            print(
                "expected broadcast_duplicate_idempotent_go case in parity matrix report",
                file=sys.stderr,
            )
            return 1

        roadmap_doc_text = ROADMAP_DOC.read_text(encoding="utf-8")
        ci_strategy_doc_text = CI_STRATEGY_DOC.read_text(encoding="utf-8")
        if "check_nonce_broadcast_parity_policy.py" not in roadmap_doc_text:
            print(
                "expected Kolme roadmap doc to reference nonce/broadcast parity policy checker command",
                file=sys.stderr,
            )
            return 1
        if "run_nonce_broadcast_parity_matrix.py" not in roadmap_doc_text:
            print(
                "expected Kolme roadmap doc to reference nonce/broadcast parity matrix command",
                file=sys.stderr,
            )
            return 1
        if "run_nonce_broadcast_parity_contract_lane.sh" not in roadmap_doc_text:
            print(
                "expected Kolme roadmap doc to reference nonce/broadcast parity contract lane command",
                file=sys.stderr,
            )
            return 1
        if "fixtures/kolme_commit/nonce_broadcast_parity_cases.json" not in roadmap_doc_text:
            print(
                "expected Kolme roadmap doc to reference nonce/broadcast parity fixture path",
                file=sys.stderr,
            )
            return 1
        if "test_run_nonce_broadcast_parity_contract_lane.sh" not in ci_strategy_doc_text:
            print(
                "expected CI strategy doc to reference nonce/broadcast parity contract lane test command",
                file=sys.stderr,
            )
            return 1
        if "KAMN_KOLME_NONCE_BROADCAST_PARITY_MAX_SECONDS=60" not in ci_strategy_doc_text:
            print(
                "expected CI strategy doc to include nonce/broadcast parity runtime budget marker",
                file=sys.stderr,
            )
            return 1

        elapsed_seconds = int(time.monotonic() - start_epoch)
        if elapsed_seconds > max_seconds:
            print(
                f"Kolme nonce/broadcast parity contract lane exceeded runtime budget: {elapsed_seconds}s",
                file=sys.stderr,
            )
            return 1
    finally:
        tmp_report.unlink(missing_ok=True)

    print("Kolme nonce/broadcast parity contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
