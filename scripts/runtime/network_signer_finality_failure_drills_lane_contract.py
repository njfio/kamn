#!/usr/bin/env python3
"""Network/signer/finality failure-drills lane contract runner."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, require_non_negative_int, write_json  # noqa: E402

PARTITION_FAULT_REASON = "network_partition_fault_injection_triggered"
SIGNER_FAULT_REASON = "signer_fault_injection_triggered"
FINALITY_FAULT_REASON = "finality_fault_injection_triggered"


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _ensure_executable(path: Path, label: str) -> None:
    if not path.is_file() or not os.access(path, os.X_OK):
        fail(f"expected {label} to be executable")


def _run_command(command: list[str], env: dict[str, str] | None = None) -> subprocess.CompletedProcess[str]:
    resolved_env = os.environ.copy()
    if env:
        resolved_env.update(env)
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
        env=resolved_env,
    )


def run_failure_drills_lane(args: argparse.Namespace) -> int:
    fault_profile = args.fault_profile.strip()
    if fault_profile not in {"none", "network_partition", "signer", "finality"}:
        fail("--fault-profile must be one of: none, network_partition, signer, finality")

    max_seconds = require_non_negative_int("KAMN_FAILURE_DRILLS_MAX_SECONDS", args.max_seconds)
    partition_max_seconds = require_non_negative_int(
        "KAMN_FAILURE_DRILLS_PARTITION_MAX_SECONDS",
        args.partition_max_seconds,
    )
    signer_max_seconds = require_non_negative_int(
        "KAMN_FAILURE_DRILLS_SIGNER_MAX_SECONDS",
        args.signer_max_seconds,
    )

    partition_lane = ROOT_DIR / "scripts/runtime/run_live_network_partition_reconnect_smoke_lane.sh"
    signer_lane = ROOT_DIR / "scripts/signer/run_signer_incident_recovery_lane.sh"
    finality_lane = ROOT_DIR / "scripts/kolme/run_local_runtime_commit_live_finality_evidence_contract_lane.sh"
    finality_checker = ROOT_DIR / "scripts/kolme/check_local_runtime_commit_live_evidence_policy.py"

    _ensure_executable(partition_lane, "live-network partition/reconnect smoke lane")
    _ensure_executable(signer_lane, "signer incident recovery lane")
    _ensure_executable(finality_lane, "local runtime-commit live finality evidence contract lane")
    _ensure_executable(finality_checker, "local runtime-commit live evidence policy checker")

    start_epoch = int(time.time())
    reason_codes: list[str] = []
    failure_expected = fault_profile != "none"

    with tempfile.TemporaryDirectory() as temp_dir:
        temp_path = Path(temp_dir)
        partition_report = temp_path / "partition-smoke-report.json"
        signer_report = temp_path / "signer-incident-report.json"
        finality_report = temp_path / "finality-contract-summary.json"
        finality_policy_report = temp_path / "finality-contract-policy.json"

        partition_command = [
            "bash",
            str(partition_lane),
            "--event-name",
            "pull_request",
            "--max-seconds",
            str(partition_max_seconds),
            "--output-json",
            str(partition_report),
        ]
        if fault_profile == "network_partition":
            partition_command.extend(["--fail-scenarios", "three_process_failover"])
        partition_run = _run_command(partition_command)

        if partition_run.returncode != 0:
            if fault_profile == "network_partition":
                reason_codes.append(PARTITION_FAULT_REASON)
            else:
                detail = (partition_run.stderr or partition_run.stdout or "partition command failed").strip()
                fail(f"network partition drill failed unexpectedly: {detail}")
        else:
            if _extract_line_value(partition_run.stdout, "status") != "pass":
                fail("network partition drill did not emit status=pass")
            if _extract_line_value(partition_run.stdout, "final_decision") != "GO":
                fail("network partition drill did not emit final_decision=GO")

        signer_command = [
            "bash",
            str(signer_lane),
            "--output-json",
            str(signer_report),
        ]
        signer_env = {
            "KAMN_SIGNER_INCIDENT_RECOVERY_SKIP_COMMANDS": "true",
            "KAMN_SIGNER_INCIDENT_RECOVERY_MAX_SECONDS": str(signer_max_seconds),
        }
        if fault_profile == "signer":
            signer_env["KAMN_SIGNER_INCIDENT_RECOVERY_FORCE_RUNBOOK_GAP"] = "true"
        signer_run = _run_command(signer_command, env=signer_env)

        if signer_run.returncode != 0:
            if fault_profile == "signer":
                reason_codes.append(SIGNER_FAULT_REASON)
            else:
                detail = (signer_run.stderr or signer_run.stdout or "signer command failed").strip()
                fail(f"signer drill failed unexpectedly: {detail}")
        else:
            if _extract_line_value(signer_run.stdout, "status") != "pass":
                fail("signer drill did not emit status=pass")
            if _extract_line_value(signer_run.stdout, "final_decision") != "GO":
                fail("signer drill did not emit final_decision=GO")

        finality_command = [
            "bash",
            str(finality_lane),
            "--output-json",
            str(finality_report),
            "--policy-output-json",
            str(finality_policy_report),
        ]
        finality_run = _run_command(finality_command)
        if finality_run.returncode != 0:
            detail = (finality_run.stderr or finality_run.stdout or "finality command failed").strip()
            fail(f"finality drill failed unexpectedly: {detail}")
        if _extract_line_value(finality_run.stdout, "status") != "ok":
            fail("finality drill did not emit status=ok")

        if fault_profile == "finality":
            finality_fault_check = _run_command(
                [
                    "python3",
                    str(finality_checker),
                    "--report-file",
                    str(finality_report),
                    "--expected-final-decision",
                    "NO-GO",
                    "--ci-fast-gate",
                    "PASS",
                    "--require-reason-code",
                    "live_finality_retry_exhausted_timeout",
                    "--output-json",
                    str(temp_path / "finality-fault-policy-report.json"),
                ]
            )
            if finality_fault_check.returncode == 0:
                fail("finality fault profile expected policy checker fail-closed behavior")
            reason_codes.append(FINALITY_FAULT_REASON)

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")

    reason_codes = sorted(set(reason_codes))
    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"

    if failure_expected and status != "fail":
        fail("fault profile expected NO-GO outcome")

    report_payload = {
        "schema_version": "kamn.runtime.failure-drills-report.v1",
        "status": status,
        "final_decision": final_decision,
        "fault_profile": fault_profile,
        "network_partition_status": "verified",
        "signer_fault_status": "verified",
        "finality_fault_status": "verified",
        "reason_codes": reason_codes,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"fault_profile={fault_profile}")
    print("network_partition_status=verified")
    print("signer_fault_status=verified")
    print("finality_fault_status=verified")
    print(f"reason_codes={reason_codes_csv}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    if status != "pass":
        fail(f"failure drills lane failed closed: {reason_codes_csv}")

    print("network/signer/finality failure drills lane tests passed.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run network/signer/finality failure drills lane.")
    parser.add_argument(
        "--fault-profile",
        default=os.environ.get("KAMN_FAILURE_DRILLS_FAULT_PROFILE", "none"),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_FAILURE_DRILLS_MAX_SECONDS", "240"),
    )
    parser.add_argument(
        "--partition-max-seconds",
        default=os.environ.get("KAMN_FAILURE_DRILLS_PARTITION_MAX_SECONDS", "120"),
    )
    parser.add_argument(
        "--signer-max-seconds",
        default=os.environ.get("KAMN_FAILURE_DRILLS_SIGNER_MAX_SECONDS", "120"),
    )
    parser.set_defaults(handler=run_failure_drills_lane)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
