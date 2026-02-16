#!/usr/bin/env python3
"""Contract lane runner for Kolme runtime commit replay matrix checks."""

from __future__ import annotations

import json
import os
import subprocess
import sys
import time
from pathlib import Path

ROOT_DIR = Path(__file__).resolve().parents[3]
POLICY_CHECKER = ROOT_DIR / "scripts/kolme/check_runtime_commit_replay_policy.py"
MATRIX_RUNNER = ROOT_DIR / "scripts/kolme/run_runtime_commit_replay_tamper_matrix.py"
ADAPTER_CONTRACT_LANE = ROOT_DIR / "scripts/kolme/run_runtime_commit_adapter_contract_lane.sh"
FIXTURE_FILE = ROOT_DIR / "fixtures/kolme_commit/runtime_commit_replay_tamper_cases.json"
ROADMAP_DOC = ROOT_DIR / "docs/planning/kolme-integration-roadmap.md"
GONOGO_DOC = ROOT_DIR / "docs/foundation/release-gonogo-checklist.md"
MAX_SECONDS = 60
RECOVERY_REASON_TAXONOMY_VERSION = "kamn.kolme.runtime-commit-recovery-reason-taxonomy.v1"
RECOVERY_REASON_CODES_CSV = (
    "recovery_nonce_not_monotonic,"
    "recovery_payload_hash_mismatch,"
    "recovery_receipt_not_final,"
    "recovery_replay_detected"
)


def command_output(command: list[str]) -> tuple[int, str]:
    result = subprocess.run(
        command,
        cwd=ROOT_DIR,
        check=False,
        capture_output=True,
        text=True,
    )
    return result.returncode, (result.stdout or "") + (result.stderr or "")


def output_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def main() -> int:
    if not os.access(POLICY_CHECKER, os.X_OK):
        print("expected runtime commit replay policy checker to be executable", file=sys.stderr)
        return 1
    if not os.access(MATRIX_RUNNER, os.X_OK):
        print("expected runtime commit replay matrix runner to be executable", file=sys.stderr)
        return 1
    if not os.access(ADAPTER_CONTRACT_LANE, os.X_OK):
        print("expected runtime commit adapter contract lane script to be executable", file=sys.stderr)
        return 1
    if not FIXTURE_FILE.is_file():
        print("expected runtime commit replay fixture file to exist", file=sys.stderr)
        return 1
    if not ROADMAP_DOC.is_file() or not GONOGO_DOC.is_file():
        print("expected Kolme roadmap and release go/no-go docs to exist", file=sys.stderr)
        return 1

    start_epoch = time.monotonic()
    tmp_report = Path(subprocess.check_output(["mktemp"], text=True).strip())
    try:
        go_code, go_output = command_output(
            [
                "python3",
                str(POLICY_CHECKER),
                "--operation-id",
                "op-go-lane-001",
                "--idempotency-key",
                "kolme-runtime-commit:op-go-lane-001:state:agent:1:12",
                "--receipt-provider",
                "kolme-local",
                "--expected-receipt-provider",
                "kolme-local",
                "--receipt-commit-id",
                "kolme-commit:op-go-lane-001:agent:1:12",
                "--expected-receipt-commit-id",
                "kolme-commit:op-go-lane-001:agent:1:12",
                "--nonce-monotonic",
                "true",
                "--replay-detected",
                "false",
                "--payload-hash-match",
                "true",
                "--receipt-finality",
                "FINAL",
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
            print("expected GO replay policy case to produce GO", file=sys.stderr)
            return 1
        if output_value(go_output, "recovery_reason_taxonomy_version") != RECOVERY_REASON_TAXONOMY_VERSION:
            print("expected recovery reason taxonomy version in GO replay policy output", file=sys.stderr)
            return 1
        if output_value(go_output, "recovery_reason_codes_csv") != RECOVERY_REASON_CODES_CSV:
            print("expected deterministic recovery reason taxonomy ordering in GO replay policy output", file=sys.stderr)
            return 1
        if output_value(go_output, "recovery_reason_codes_value") != "none":
            print("expected recovery reason taxonomy value=none in GO replay policy output", file=sys.stderr)
            return 1
        if output_value(go_output, "retransmission_evidence_contract_version") != "v1":
            print("expected retransmission evidence contract version in GO replay policy output", file=sys.stderr)
            return 1
        if output_value(go_output, "nonce_idempotency_contract_version") != "v1":
            print("expected nonce-idempotency contract version in GO replay policy output", file=sys.stderr)
            return 1

        no_go_code, no_go_output = command_output(
            [
                "python3",
                str(POLICY_CHECKER),
                "--operation-id",
                "op-no-go-lane-001",
                "--idempotency-key",
                "kolme-runtime-commit:op-no-go-lane-001:state:agent:2:12",
                "--receipt-provider",
                "kolme-local",
                "--expected-receipt-provider",
                "kolme-local",
                "--receipt-commit-id",
                "kolme-commit:op-no-go-lane-001:agent:2:12",
                "--expected-receipt-commit-id",
                "kolme-commit:op-no-go-lane-001:agent:2:12",
                "--nonce-monotonic",
                "false",
                "--replay-detected",
                "true",
                "--payload-hash-match",
                "true",
                "--receipt-finality",
                "FINAL",
                "--ci-fast-gate",
                "PASS",
                "--output-json",
                str(tmp_report),
            ]
        )
        if no_go_code == 0:
            print("expected replay-detected policy case to fail closed", file=sys.stderr)
            return 1
        if "final_decision=NO-GO" not in no_go_output:
            print("expected replay-detected policy case to produce NO-GO", file=sys.stderr)
            return 1
        if output_value(no_go_output, "recovery_reason_taxonomy_version") != RECOVERY_REASON_TAXONOMY_VERSION:
            print("expected recovery reason taxonomy version in NO-GO replay policy output", file=sys.stderr)
            return 1
        if output_value(no_go_output, "recovery_reason_codes_csv") != RECOVERY_REASON_CODES_CSV:
            print("expected deterministic recovery reason taxonomy ordering in NO-GO replay policy output", file=sys.stderr)
            return 1
        expected_no_go_recovery_codes = (
            "recovery_nonce_not_monotonic,"
            "recovery_replay_detected"
        )
        if output_value(no_go_output, "recovery_reason_codes_value") != expected_no_go_recovery_codes:
            print("expected deterministic recovery reason taxonomy value in NO-GO replay policy output", file=sys.stderr)
            return 1
        if output_value(no_go_output, "retransmission_evidence_contract_version") != "v1":
            print("expected retransmission evidence contract version in NO-GO replay policy output", file=sys.stderr)
            return 1
        if output_value(no_go_output, "nonce_idempotency_contract_version") != "v1":
            print("expected nonce-idempotency contract version in NO-GO replay policy output", file=sys.stderr)
            return 1

        replay_policy_payload = json.loads(tmp_report.read_text(encoding="utf-8"))
        if replay_policy_payload.get("schema_version") != "kamn.kolme.runtime-commit-replay-policy-report.v1":
            print("unexpected runtime commit replay policy report schema", file=sys.stderr)
            return 1
        if replay_policy_payload.get("recovery_reason_taxonomy_version") != RECOVERY_REASON_TAXONOMY_VERSION:
            print("expected recovery reason taxonomy version in replay policy report", file=sys.stderr)
            return 1
        if replay_policy_payload.get("recovery_reason_codes_csv") != RECOVERY_REASON_CODES_CSV:
            print("expected recovery reason taxonomy ordering in replay policy report", file=sys.stderr)
            return 1
        if replay_policy_payload.get("recovery_reason_codes_value") != expected_no_go_recovery_codes:
            print("expected deterministic recovery reason taxonomy value in replay policy report", file=sys.stderr)
            return 1
        if replay_policy_payload.get("retransmission_evidence_contract_version") != "v1":
            print("expected retransmission evidence contract version in replay policy report", file=sys.stderr)
            return 1
        if replay_policy_payload.get("nonce_idempotency_contract_version") != "v1":
            print("expected nonce-idempotency contract version in replay policy report", file=sys.stderr)
            return 1

        matrix_code, matrix_output = command_output(
            [
                "python3",
                str(MATRIX_RUNNER),
                "--fixture",
                str(FIXTURE_FILE),
                "--max-cases",
                "3",
                "--output-json",
                str(tmp_report),
            ]
        )
        if matrix_code != 0:
            print(matrix_output, file=sys.stderr)
            return matrix_code
        if "status=pass;" not in matrix_output:
            print("expected runtime commit replay matrix to pass for fixture cases", file=sys.stderr)
            return 1

        adapter_code, adapter_output = command_output(
            ["bash", str(ADAPTER_CONTRACT_LANE)]
        )
        if adapter_code != 0:
            print(adapter_output, file=sys.stderr)
            return adapter_code
        if "Kolme runtime commit adapter contract lane tests passed." not in adapter_output:
            print("expected runtime commit adapter contract lane success marker", file=sys.stderr)
            return 1

        matrix_payload = json.loads(tmp_report.read_text(encoding="utf-8"))
        if matrix_payload.get("schema_version") != "kamn.kolme.runtime-commit-replay-matrix.v1":
            print("unexpected runtime commit replay matrix report schema", file=sys.stderr)
            return 1
        if matrix_payload.get("status") != "pass":
            print("expected runtime commit replay matrix report to pass", file=sys.stderr)
            return 1
        cases = matrix_payload.get("cases", [])
        if not any(case.get("case_id") == "no_go_replay_detected" for case in cases):
            print(
                "expected no_go_replay_detected case in runtime commit replay matrix report",
                file=sys.stderr,
            )
            return 1

        roadmap_doc_text = ROADMAP_DOC.read_text(encoding="utf-8")
        gonogo_doc_text = GONOGO_DOC.read_text(encoding="utf-8")
        if "check_runtime_commit_replay_policy.py" not in roadmap_doc_text:
            print(
                "expected Kolme integration roadmap to reference runtime commit replay policy checker command",
                file=sys.stderr,
            )
            return 1
        if "run_runtime_commit_replay_tamper_matrix.py" not in roadmap_doc_text:
            print(
                "expected Kolme integration roadmap to reference runtime commit replay matrix command",
                file=sys.stderr,
            )
            return 1
        if "run_runtime_commit_replay_contract_lane.sh" not in roadmap_doc_text:
            print(
                "expected Kolme integration roadmap to reference runtime commit replay contract lane command",
                file=sys.stderr,
            )
            return 1
        if RECOVERY_REASON_TAXONOMY_VERSION not in roadmap_doc_text:
            print(
                "expected Kolme integration roadmap to reference runtime commit recovery reason taxonomy marker",
                file=sys.stderr,
            )
            return 1
        if "retransmission_evidence_contract_version" not in roadmap_doc_text:
            print(
                "expected Kolme integration roadmap to reference retransmission evidence contract marker",
                file=sys.stderr,
            )
            return 1
        if "nonce_idempotency_contract_version" not in roadmap_doc_text:
            print(
                "expected Kolme integration roadmap to reference nonce idempotency contract marker",
                file=sys.stderr,
            )
            return 1
        if "run_runtime_commit_adapter_contract_lane.sh" not in roadmap_doc_text:
            print(
                "expected Kolme integration roadmap to reference runtime commit adapter contract lane command",
                file=sys.stderr,
            )
            return 1
        if "run_runtime_commit_replay_tamper_matrix.py" not in gonogo_doc_text:
            print(
                "expected release go/no-go doc to reference runtime commit replay matrix command",
                file=sys.stderr,
            )
            return 1
        if "run_runtime_commit_adapter_contract_lane.sh" not in gonogo_doc_text:
            print(
                "expected release go/no-go doc to reference runtime commit adapter contract lane command",
                file=sys.stderr,
            )
            return 1
        if RECOVERY_REASON_TAXONOMY_VERSION not in gonogo_doc_text:
            print(
                "expected release go/no-go doc to reference runtime commit recovery reason taxonomy marker",
                file=sys.stderr,
            )
            return 1
        if "recovery_reason_codes_csv" not in gonogo_doc_text:
            print(
                "expected release go/no-go doc to reference recovery reason taxonomy ordering marker",
                file=sys.stderr,
            )
            return 1
        if "retransmission_evidence_contract_version" not in gonogo_doc_text:
            print(
                "expected release go/no-go doc to reference retransmission evidence contract marker",
                file=sys.stderr,
            )
            return 1

        elapsed_seconds = int(time.monotonic() - start_epoch)
        if elapsed_seconds > MAX_SECONDS:
            print(
                f"Kolme runtime commit replay contract lane exceeded runtime budget: {elapsed_seconds}s",
                file=sys.stderr,
            )
            return 1
    finally:
        tmp_report.unlink(missing_ok=True)

    print("Kolme runtime commit replay contract lane tests passed.")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
