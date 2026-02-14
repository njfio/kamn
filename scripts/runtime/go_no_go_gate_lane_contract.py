#!/usr/bin/env python3
"""Production go/no-go gate lane contract runner."""

from __future__ import annotations

import argparse
import json
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

GATE_DECISION_FAULT_REASON = "gate_decision_fault_injection_triggered"


def _ensure_executable(path: Path, label: str) -> None:
    if not path.is_file() or not os.access(path, os.X_OK):
        fail(f"expected {label} to be executable")


def _run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )


def run_go_no_go_gate_lane(args: argparse.Namespace) -> int:
    fault_profile = args.fault_profile.strip()
    if fault_profile not in {"none", "gate_decision"}:
        fail("--fault-profile must be one of: none, gate_decision")

    max_seconds = require_non_negative_int("KAMN_GONOGO_GATE_MAX_SECONDS", args.max_seconds)
    start_epoch = int(time.time())

    gonogo_deep_lane = ROOT_DIR / "scripts/deploy/run_gonogo_evidence_deep_lane.sh"
    rollback_lane = ROOT_DIR / "scripts/deploy/run_deployment_slo_rollback_contract_lane.sh"
    dr_lane = ROOT_DIR / "scripts/deploy/run_dr_evidence_contract_lane.sh"
    gonogo_generator = ROOT_DIR / "scripts/deploy/generate_gonogo_evidence_bundle.sh"
    gonogo_checker = ROOT_DIR / "scripts/deploy/check_gonogo_evidence_policy.sh"

    _ensure_executable(gonogo_deep_lane, "go/no-go evidence deep lane")
    _ensure_executable(rollback_lane, "deployment slo rollback contract lane")
    _ensure_executable(dr_lane, "dr evidence contract lane")
    _ensure_executable(gonogo_generator, "go/no-go evidence bundle generator")
    _ensure_executable(gonogo_checker, "go/no-go evidence policy checker")

    reason_codes: list[str] = []

    gonogo_run = _run_command(["bash", str(gonogo_deep_lane)])
    if gonogo_run.returncode != 0:
        detail = (gonogo_run.stderr or gonogo_run.stdout or "go/no-go deep lane failed").strip()
        fail(f"go/no-go evidence lane failed unexpectedly: {detail}")
    if "go/no-go evidence deep lane tests passed." not in gonogo_run.stdout:
        fail("go/no-go evidence lane did not emit success marker")

    rollback_run = _run_command(["bash", str(rollback_lane)])
    if rollback_run.returncode != 0:
        detail = (rollback_run.stderr or rollback_run.stdout or "rollback lane failed").strip()
        fail(f"rollback readiness lane failed unexpectedly: {detail}")
    if "final_decision=GO" not in rollback_run.stdout:
        fail("rollback readiness lane did not emit final_decision=GO")

    dr_run = _run_command(["bash", str(dr_lane)])
    if dr_run.returncode != 0:
        detail = (dr_run.stderr or dr_run.stdout or "dr lane failed").strip()
        fail(f"dr readiness lane failed unexpectedly: {detail}")
    if "dr evidence contract lane tests passed." not in dr_run.stdout:
        fail("dr readiness lane did not emit success marker")

    if fault_profile == "gate_decision":
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            no_go_bundle = temp_path / "gonogo-no-go-bundle.json"
            drift_bundle = temp_path / "gonogo-drift-bundle.json"

            generate_run = _run_command(
                [
                    "bash",
                    str(gonogo_generator),
                    "--output-file",
                    str(no_go_bundle),
                    "--release-candidate",
                    "v1.0.0-fault",
                    "--schema-target-version",
                    "1.0.0",
                    "--runtime-image-digest",
                    "sha256:fault",
                    "--ci-fast-gate",
                    "PASS",
                    "--ci-deep-lane",
                    "FAIL",
                    "--rollback-precheck",
                    "PASS",
                    "--rollback-trigger-status",
                    "CLEAR",
                    "--required-approvals",
                    "2",
                    "--received-approvals",
                    "1",
                ]
            )
            if generate_run.returncode != 0:
                detail = (generate_run.stderr or generate_run.stdout or "generator failed").strip()
                fail(f"go/no-go generator failed unexpectedly: {detail}")

            payload = json.loads(no_go_bundle.read_text(encoding="utf-8"))
            payload["final_decision"] = "GO"
            drift_bundle.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

            drift_run = _run_command(
                [
                    "bash",
                    str(gonogo_checker),
                    "--bundle-file",
                    str(drift_bundle),
                ]
            )
            if drift_run.returncode == 0:
                fail("gate_decision fault profile expected policy checker fail-closed behavior")
            reason_codes.append(GATE_DECISION_FAULT_REASON)

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")

    reason_codes = sorted(set(reason_codes))
    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"

    report_payload = {
        "schema_version": "kamn.runtime.go-no-go-gate-report.v1",
        "status": status,
        "final_decision": final_decision,
        "fault_profile": fault_profile,
        "go_no_go_evidence_status": "verified",
        "rollback_readiness_status": "verified",
        "dr_readiness_status": "verified",
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
    print("go_no_go_evidence_status=verified")
    print("rollback_readiness_status=verified")
    print("dr_readiness_status=verified")
    print(f"reason_codes={reason_codes_csv}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    if status != "pass":
        fail(f"go/no-go gate lane failed closed: {reason_codes_csv}")

    print("go/no-go gate lane tests passed.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run production go/no-go gate lane.")
    parser.add_argument(
        "--fault-profile",
        default=os.environ.get("KAMN_GONOGO_GATE_FAULT_PROFILE", "none"),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_GONOGO_GATE_MAX_SECONDS", "180"),
    )
    parser.set_defaults(handler=run_go_no_go_gate_lane)
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
