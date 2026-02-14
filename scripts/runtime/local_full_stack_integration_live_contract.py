#!/usr/bin/env python3
"""Local full-stack integration live-validation lane and policy checker."""

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

from framework.contract_framework import (  # noqa: E402
    ContractError,
    DecisionAccumulator,
    fail,
    load_json,
    require_enum,
    require_positive_int,
    write_json,
)

RUN_LANE_SCHEMA = "kamn.runtime.local-full-stack-integration-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.local-full-stack-integration-live-policy-report.v1"
EVIDENCE_BUNDLE_SCHEMA = "kamn.runtime.local-full-stack-integration-evidence-bundle.v1"
OPT_IN_ENV = "KAMN_LOCAL_FULL_STACK_INTEGRATION_OPT_IN"
DRY_RUN_REASON = "dry_run_no_commands_executed"
RUN_REASON = "local_full_stack_integration_live_validation_executed"
FAST_GATE_EXCLUSION_REASON = "local_full_stack_integration_run_mode_excluded_from_fast_gate"


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _run_command(
    command: list[str],
    *,
    timeout_seconds: int,
    env: dict[str, str] | None = None,
) -> str:
    completed = subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
        timeout=timeout_seconds,
        env=env,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"lane command failed: {' '.join(command)}: {detail}")
    return completed.stdout


def _write_json(path: Path, payload: dict[str, object]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")


def run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    max_seconds = require_positive_int("KAMN_LOCAL_FULL_STACK_INTEGRATION_MAX_SECONDS", args.max_seconds)
    command_max_seconds = require_positive_int(
        "KAMN_LOCAL_FULL_STACK_INTEGRATION_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    if mode == "run" and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            "KAMN_LOCAL_FULL_STACK_INTEGRATION_OPT_IN=1"
        )

    start_epoch = int(time.time())
    commands_executed = 0
    artifact_paths: dict[str, str] = {}
    if mode == "run":
        artifact_dir = Path(tempfile.mkdtemp(prefix="local-full-stack-integration-live-"))
        full_io_report = artifact_dir / "full-io-scenario-matrix-report.json"
        full_runtime_report = artifact_dir / "local-full-runtime-report.json"
        evidence_bundle_file = artifact_dir / "local-full-stack-evidence-bundle.json"

        full_io_output = _run_command(
            [
                "bash",
                "scripts/runtime/validate_full_io_scenario_matrix_live.sh",
                "--mode",
                "run",
                "--ci-fast-gate",
                "FAIL",
                "--max-seconds",
                str(command_max_seconds),
                "--output-json",
                str(full_io_report),
            ],
            timeout_seconds=command_max_seconds,
            env={**os.environ, "KAMN_LOCAL_FULL_IO_SCENARIO_MATRIX_OPT_IN": "1"},
        )
        if _extract_line_value(full_io_output, "status") != "pass":
            fail("full I/O scenario matrix command did not emit status=pass")
        if _extract_line_value(full_io_output, "final_decision") != "GO":
            fail("full I/O scenario matrix command did not emit final_decision=GO")
        commands_executed += 1

        full_runtime_output = _run_command(
            [
                "bash",
                "scripts/runtime/validate_local_full_runtime_live.sh",
                "--mode",
                "run",
                "--ci-fast-gate",
                "FAIL",
                "--max-seconds",
                str(command_max_seconds),
                "--command-max-seconds",
                str(min(command_max_seconds, 180)),
                "--output-json",
                str(full_runtime_report),
            ],
            timeout_seconds=command_max_seconds,
            env={**os.environ, "KAMN_LOCAL_FULL_RUNTIME_LIVE_OPT_IN": "1"},
        )
        if _extract_line_value(full_runtime_output, "status") != "pass":
            fail("local full-runtime command did not emit status=pass")
        if _extract_line_value(full_runtime_output, "final_decision") != "GO":
            fail("local full-runtime command did not emit final_decision=GO")
        commands_executed += 1

        full_io_payload = json.loads(full_io_report.read_text(encoding="utf-8"))
        full_runtime_payload = json.loads(full_runtime_report.read_text(encoding="utf-8"))
        if full_io_payload.get("final_decision") != "GO":
            fail("full I/O scenario matrix report missing final_decision=GO")
        if full_runtime_payload.get("final_decision") != "GO":
            fail("local full-runtime report missing final_decision=GO")

        evidence_bundle = {
            "schema_version": EVIDENCE_BUNDLE_SCHEMA,
            "status": "pass",
            "final_decision": "GO",
            "lane_mode": mode,
            "full_io_matrix_report_file": str(full_io_report),
            "full_runtime_report_file": str(full_runtime_report),
            "commands_executed": commands_executed,
            "ci_fast_gate_eligibility": "excluded_local_heavy",
        }
        _write_json(evidence_bundle_file, evidence_bundle)

        artifact_paths = {
            "full_io_matrix_report": str(full_io_report),
            "full_runtime_report": str(full_runtime_report),
            "evidence_bundle_file": str(evidence_bundle_file),
        }

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "local full-stack integration lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    run_mode_command_status = "executed" if mode == "run" else "dry_run_no_commands_executed"
    ci_fast_gate_eligibility = "excluded_local_heavy" if mode == "run" else "eligible"
    reason_code = RUN_REASON if mode == "run" else DRY_RUN_REASON

    payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "ci_fast_gate": ci_fast_gate,
        "ci_fast_gate_eligibility": ci_fast_gate_eligibility,
        "fast_gate_exclusion_status": "verified",
        "fast_gate_exclusion_reason_code": FAST_GATE_EXCLUSION_REASON,
        "scenario_matrix_status": "verified",
        "full_runtime_status": "verified",
        "evidence_bundle_status": "verified",
        "run_mode_command_status": run_mode_command_status,
        "run_mode_command_count": commands_executed,
        "reason_code": reason_code,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
        "command_max_seconds": command_max_seconds,
        "artifact_paths": artifact_paths,
    }
    if args.output_json:
        write_json(Path(args.output_json), payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(f"ci_fast_gate_eligibility={ci_fast_gate_eligibility}")
    print("fast_gate_exclusion_status=verified")
    print(f"fast_gate_exclusion_reason_code={FAST_GATE_EXCLUSION_REASON}")
    print("scenario_matrix_status=verified")
    print("full_runtime_status=verified")
    print("evidence_bundle_status=verified")
    print(f"run_mode_command_status={run_mode_command_status}")
    print(f"run_mode_command_count={commands_executed}")
    print(f"reason_code={reason_code}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
    return 0


def check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file)
    if not report_file.is_file():
        fail(f"report file does not exist: {report_file}")

    expected_final_decision = require_enum(
        "--expected-final-decision",
        args.expected_final_decision.strip(),
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate.strip(), ("PASS", "FAIL"))
    payload = load_json(report_file)

    checks = DecisionAccumulator()
    checks.reject_if(
        payload.get("schema_version") != RUN_LANE_SCHEMA,
        "local_full_stack_integration_policy_schema_mismatch",
    )
    checks.reject_if(payload.get("status") != "pass", "local_full_stack_integration_policy_status_mismatch")
    checks.reject_if(
        payload.get("final_decision") != "GO",
        "local_full_stack_integration_policy_final_decision_mismatch",
    )
    checks.reject_if(
        payload.get("ci_fast_gate") != ci_fast_gate,
        "local_full_stack_integration_policy_ci_fast_gate_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_status") != "verified",
        "local_full_stack_integration_policy_fast_gate_exclusion_mismatch",
    )
    checks.reject_if(
        payload.get("fast_gate_exclusion_reason_code") != FAST_GATE_EXCLUSION_REASON,
        "local_full_stack_integration_policy_fast_gate_reason_mismatch",
    )
    checks.reject_if(
        payload.get("scenario_matrix_status") != "verified",
        "local_full_stack_integration_policy_scenario_matrix_status_mismatch",
    )
    checks.reject_if(
        payload.get("full_runtime_status") != "verified",
        "local_full_stack_integration_policy_full_runtime_status_mismatch",
    )
    checks.reject_if(
        payload.get("evidence_bundle_status") != "verified",
        "local_full_stack_integration_policy_evidence_bundle_status_mismatch",
    )

    lane_mode = payload.get("lane_mode")
    checks.reject_if(
        lane_mode not in ("dry-run", "run"),
        "local_full_stack_integration_policy_lane_mode_invalid",
    )
    command_count = payload.get("run_mode_command_count")
    checks.reject_if(
        not isinstance(command_count, int) or command_count < 0,
        "local_full_stack_integration_policy_command_count_invalid",
    )
    command_status = payload.get("run_mode_command_status")
    reason_code = payload.get("reason_code")
    artifact_paths = payload.get("artifact_paths")
    checks.reject_if(
        not isinstance(artifact_paths, dict),
        "local_full_stack_integration_policy_artifact_paths_invalid",
    )

    if lane_mode == "dry-run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "eligible",
            "local_full_stack_integration_policy_dry_run_eligibility_mismatch",
        )
        checks.reject_if(
            command_count != 0,
            "local_full_stack_integration_policy_dry_run_command_count_mismatch",
        )
        checks.reject_if(
            command_status != "dry_run_no_commands_executed",
            "local_full_stack_integration_policy_dry_run_command_status_mismatch",
        )
        checks.reject_if(
            reason_code != DRY_RUN_REASON,
            "local_full_stack_integration_policy_dry_run_reason_code_mismatch",
        )
    elif lane_mode == "run":
        checks.reject_if(
            payload.get("ci_fast_gate_eligibility") != "excluded_local_heavy",
            "local_full_stack_integration_policy_run_mode_exclusion_mismatch",
        )
        checks.reject_if(
            command_count < 2,
            "local_full_stack_integration_policy_run_mode_command_count_mismatch",
        )
        checks.reject_if(
            command_status != "executed",
            "local_full_stack_integration_policy_run_mode_command_status_mismatch",
        )
        checks.reject_if(
            reason_code != RUN_REASON,
            "local_full_stack_integration_policy_run_mode_reason_code_mismatch",
        )
        required_artifacts = (
            "full_io_matrix_report",
            "full_runtime_report",
            "evidence_bundle_file",
        )
        if isinstance(artifact_paths, dict):
            for artifact_key in required_artifacts:
                artifact_value = artifact_paths.get(artifact_key)
                checks.reject_if(
                    not isinstance(artifact_value, str) or not Path(artifact_value).is_file(),
                    f"local_full_stack_integration_policy_artifact_missing:{artifact_key}",
                )

    observed_final_decision, decision_reasons = checks.finalize(
        "local_full_stack_integration_policy_verified"
    )
    failed_checks: list[str] = []
    if observed_final_decision == "NO-GO":
        failed_checks.extend(decision_reasons)
    if observed_final_decision != expected_final_decision:
        failed_checks.append("local_full_stack_integration_policy_expected_decision_mismatch")

    report_payload = {
        "schema_version": POLICY_SCHEMA,
        "status": "ok" if not failed_checks else "fail",
        "final_decision": observed_final_decision,
        "expected_final_decision": expected_final_decision,
        "ci_fast_gate": ci_fast_gate,
        "decision_reasons": decision_reasons,
        "local_full_stack_integration_policy_status": "verified" if not failed_checks else "failed",
        "failed_checks": failed_checks,
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    print(f"status={'ok' if not failed_checks else 'fail'}")
    print(f"final_decision={observed_final_decision}")
    print(f"expected_final_decision={expected_final_decision}")
    print(f"ci_fast_gate={ci_fast_gate}")
    print(
        "local_full_stack_integration_policy_status="
        f"{'verified' if not failed_checks else 'failed'}"
    )
    print(f"failed_checks={','.join(failed_checks)}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    if failed_checks:
        fail(",".join(failed_checks))
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Local full-stack integration lane contracts.")
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser("run-lane", help="Run local full-stack integration lane.")
    run_lane_parser.add_argument("--mode", default=os.environ.get("KAMN_LOCAL_FULL_STACK_INTEGRATION_MODE", "dry-run"))
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_FULL_STACK_INTEGRATION_MAX_SECONDS", "360"),
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get("KAMN_LOCAL_FULL_STACK_INTEGRATION_COMMAND_MAX_SECONDS", "300"),
    )
    run_lane_parser.add_argument("--ci-fast-gate", default=os.environ.get("KAMN_CI_FAST_GATE", "PASS"))
    run_lane_parser.add_argument("--local-opt-in", default=os.environ.get(OPT_IN_ENV, "0"))
    run_lane_parser.add_argument("--output-json", default="")
    run_lane_parser.set_defaults(handler=run_lane)

    policy_parser = subparsers.add_parser("check-policy", help="Check local full-stack integration policy.")
    policy_parser.add_argument("--report-file", required=True)
    policy_parser.add_argument("--expected-final-decision", default="GO")
    policy_parser.add_argument("--ci-fast-gate", default="PASS")
    policy_parser.add_argument("--output-json", default="")
    policy_parser.set_defaults(handler=check_policy)
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
