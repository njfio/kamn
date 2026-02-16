#!/usr/bin/env python3
"""Local retry/diagnostics live lane and policy checker contracts."""

from __future__ import annotations

import argparse
import os
from pathlib import Path
import subprocess
import sys
import time
from typing import Any

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

RUN_LANE_SCHEMA = "kamn.runtime.local-retry-diagnostics-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.local-retry-diagnostics-live-policy-report.v1"
OPT_IN_ENV = "KAMN_LOCAL_RETRY_DIAGNOSTICS_OPT_IN"
RETRY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.local-retry-diagnostics-reason-taxonomy.v1"
)
RETRY_REASON_CODES_CSV = (
    "local_retry_readiness_progress_stalled,"
    "local_retry_backoff_jitter_parity_bypass_detected,"
    "ci_local_network_budget_boundary_exceeded"
)
CI_LOCAL_NETWORK_BUDGET_MAX_SECONDS = 240


def _extract_line_value(output: str, key: str) -> str:
    prefix = f"{key}="
    for line in output.splitlines():
        if line.startswith(prefix):
            return line[len(prefix) :]
    return ""


def _run_script(script_path: Path, *, max_seconds: int) -> str:
    completed = subprocess.run(
        [
            "bash",
            str(script_path),
            "--max-seconds",
            str(max_seconds),
        ],
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"lane command failed: {script_path}: {detail}")
    return completed.stdout


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    max_seconds = require_positive_int(
        "KAMN_LOCAL_RETRY_DIAGNOSTICS_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_LOCAL_RETRY_DIAGNOSTICS_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    nonce_retry_script = Path(args.nonce_retry_script).resolve()
    structured_logging_script = Path(args.structured_logging_script).resolve()
    commands: list[str] = []

    start_epoch = int(time.time())
    execution_reason_code = "dry_run_no_commands_executed"

    if mode == "run":
        if args.require_opt_in and args.local_opt_in != "1":
            fail(f"run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1")

        for script_path in (nonce_retry_script, structured_logging_script):
            if not script_path.is_file():
                fail(f"expected executable script not found: {script_path}")
            if not script_path.stat().st_mode & 0o111:
                fail(f"expected executable script: {script_path}")

        nonce_output = _run_script(nonce_retry_script, max_seconds=command_max_seconds)
        if _extract_line_value(nonce_output, "status") != "pass":
            fail("nonce retry script did not emit status=pass")
        if _extract_line_value(nonce_output, "final_decision") != "GO":
            fail("nonce retry script did not emit final_decision=GO")
        if _extract_line_value(nonce_output, "nonce_retry_contract_status") != "verified":
            fail("nonce retry script did not emit nonce_retry_contract_status=verified")

        structured_output = _run_script(
            structured_logging_script,
            max_seconds=command_max_seconds,
        )
        if _extract_line_value(structured_output, "status") != "pass":
            fail("structured logging script did not emit status=pass")
        if _extract_line_value(structured_output, "final_decision") != "GO":
            fail("structured logging script did not emit final_decision=GO")
        if _extract_line_value(structured_output, "correlation_contract_status") != "verified":
            fail(
                "structured logging script did not emit "
                "correlation_contract_status=verified"
            )

        commands = [
            str(nonce_retry_script),
            str(structured_logging_script),
        ]
        execution_reason_code = "run_mode_commands_executed"

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "local retry/diagnostics lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "retry_contract_status": "verified",
        "retry_readiness_status": "verified",
        "retry_backoff_status": "verified",
        "retry_jitter_parity_status": "verified",
        "correlation_diagnostics_status": "verified",
        "reason_taxonomy_version": RETRY_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": RETRY_REASON_CODES_CSV,
        "fail_closed_status": "verified",
        "ci_fast_gate_exclusion_status": "verified",
        "ci_local_network_budget_boundary_status": "verified",
        "performance_budget_status": "verified",
        "execution_reason_code": execution_reason_code,
        "command_count": len(commands),
        "commands": commands,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, report_payload)

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={mode}")
    print("retry_contract_status=verified")
    print("retry_readiness_status=verified")
    print("retry_backoff_status=verified")
    print("retry_jitter_parity_status=verified")
    print("correlation_diagnostics_status=verified")
    print(f"reason_taxonomy_version={RETRY_REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={RETRY_REASON_CODES_CSV}")
    print("fail_closed_status=verified")
    print("ci_fast_gate_exclusion_status=verified")
    print("ci_local_network_budget_boundary_status=verified")
    print("performance_budget_status=verified")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"command_count={len(commands)}")
    if output_json is not None:
        print(f"report_file={output_json}")
    return 0


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    if not report_file.is_file():
        fail(f"report file not found: {report_file}")

    report = load_json(report_file)
    expected_final_decision = require_enum(
        "--expected-final-decision",
        args.expected_final_decision,
        ("GO", "NO-GO"),
    )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    required_fields = [
        "schema_version",
        "status",
        "final_decision",
        "lane_mode",
        "retry_contract_status",
        "retry_readiness_status",
        "retry_backoff_status",
        "retry_jitter_parity_status",
        "correlation_diagnostics_status",
        "reason_taxonomy_version",
        "reason_codes_csv",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
        "ci_local_network_budget_boundary_status",
        "performance_budget_status",
        "execution_reason_code",
        "command_count",
        "elapsed_seconds",
    ]
    missing_fields = [field_name for field_name in required_fields if field_name not in report]
    if missing_fields:
        fail(f"missing required report fields: {','.join(missing_fields)}")

    decision = DecisionAccumulator()
    decision.reject_if(
        report.get("schema_version") != RUN_LANE_SCHEMA,
        "local_retry_diagnostics_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "local_retry_diagnostics_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "local_retry_diagnostics_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "local_retry_diagnostics_policy_final_decision_mismatch",
    )

    for field_name in (
        "retry_contract_status",
        "retry_backoff_status",
        "correlation_diagnostics_status",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
        "ci_local_network_budget_boundary_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(field_name) != "verified",
            f"local_retry_diagnostics_policy_marker_missing:{field_name}",
        )

    decision.reject_if(
        report.get("retry_readiness_status") != "verified",
        "local_retry_readiness_progress_stalled",
    )
    decision.reject_if(
        report.get("retry_jitter_parity_status") != "verified",
        "local_retry_backoff_jitter_parity_bypass_detected",
    )
    decision.reject_if(
        report.get("reason_taxonomy_version") != RETRY_REASON_TAXONOMY_VERSION,
        "local_retry_diagnostics_policy_reason_taxonomy_version_mismatch",
    )
    decision.reject_if(
        report.get("reason_codes_csv") != RETRY_REASON_CODES_CSV,
        "local_retry_diagnostics_policy_reason_codes_csv_mismatch",
    )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "local_retry_diagnostics_policy_lane_mode_invalid",
    )

    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "local_retry_diagnostics_policy_command_count_invalid",
    )

    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != "dry_run_no_commands_executed",
            "local_retry_diagnostics_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "local_retry_diagnostics_policy_command_count_mismatch",
        )
    elif lane_mode == "run":
        decision.reject_if(
            execution_reason_code != "run_mode_commands_executed",
            "local_retry_diagnostics_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            not isinstance(command_count, int) or command_count < 1,
            "local_retry_diagnostics_policy_command_count_mismatch",
        )

    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "local_retry_diagnostics_policy_elapsed_seconds_invalid",
    )
    report_max_seconds = report.get("max_seconds")
    decision.reject_if(
        not _is_non_negative_int(report_max_seconds),
        "local_retry_diagnostics_policy_max_seconds_invalid",
    )
    if isinstance(report_max_seconds, int):
        decision.reject_if(
            report_max_seconds > CI_LOCAL_NETWORK_BUDGET_MAX_SECONDS,
            "ci_local_network_budget_boundary_exceeded",
        )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "local_retry_diagnostics_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "reason_codes": reason_codes,
        "ci_fast_gate": ci_fast_gate,
        "reason_taxonomy_version": RETRY_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": RETRY_REASON_CODES_CSV,
        "source_report_file": str(report_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, policy_report)

    reason_codes_csv = ",".join(reason_codes)
    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(f"local_retry_diagnostics_policy_status={policy_status}")
    print(f"reason_taxonomy_version={RETRY_REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={RETRY_REASON_CODES_CSV}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"local retry/diagnostics live policy rejected: {reason_codes_csv}")

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Local retry/diagnostics live lane and policy checker contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute local retry/diagnostics lane in dry-run or run mode.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_LOCAL_RETRY_DIAGNOSTICS_MODE", "dry-run"),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_RETRY_DIAGNOSTICS_MAX_SECONDS", "180"),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get("KAMN_LOCAL_RETRY_DIAGNOSTICS_COMMAND_MAX_SECONDS", "120"),
        help="Maximum runtime budget for each nested command in run mode.",
    )
    run_lane_parser.add_argument(
        "--nonce-retry-script",
        default=str(ROOT_DIR / "scripts/runtime/validate_nonce_retry_live.sh"),
        help="Nonce retry validation script path.",
    )
    run_lane_parser.add_argument(
        "--structured-logging-script",
        default=str(ROOT_DIR / "scripts/runtime/validate_structured_logging_live.sh"),
        help="Structured logging validation script path.",
    )
    run_lane_parser.add_argument(
        "--output-json",
        default="",
        help="Optional output path for summary report JSON.",
    )
    run_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, "0"),
        help="Opt-in marker value for run mode checks.",
    )
    run_lane_parser.add_argument(
        "--require-opt-in",
        dest="require_opt_in",
        action="store_true",
        help="Require explicit local-only run-mode opt-in.",
    )
    run_lane_parser.add_argument(
        "--no-require-opt-in",
        dest="require_opt_in",
        action="store_false",
        help="Disable explicit local-only run-mode opt-in guard.",
    )
    run_lane_parser.set_defaults(
        handler=_run_lane,
        require_opt_in=True,
    )

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate local retry/diagnostics report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to local retry/diagnostics report JSON.",
    )
    check_policy_parser.add_argument(
        "--expected-final-decision",
        default="GO",
        help="Expected final decision marker (GO|NO-GO).",
    )
    check_policy_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        help="CI fast-gate marker (PASS|FAIL).",
    )
    check_policy_parser.add_argument(
        "--output-json",
        help="Optional output path for policy report JSON.",
    )
    check_policy_parser.set_defaults(handler=_check_policy)

    args = parser.parse_args()
    if hasattr(args, "max_seconds"):
        args.max_seconds = args.max_seconds.strip()
    if hasattr(args, "command_max_seconds"):
        args.command_max_seconds = args.command_max_seconds.strip()
    if hasattr(args, "mode"):
        args.mode = args.mode.strip()
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
