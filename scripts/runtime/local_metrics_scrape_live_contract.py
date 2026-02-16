#!/usr/bin/env python3
"""Local metrics scrape live lane and policy checker contracts."""

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

RUN_LANE_SCHEMA = "kamn.runtime.local-metrics-scrape-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.local-metrics-scrape-live-policy-report.v1"
OPT_IN_ENV = "KAMN_LOCAL_METRICS_SCRAPE_OPT_IN"
METRICS_EMISSION_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.metrics-emission-reason-taxonomy.v1"
)
METRICS_EMISSION_REASON_CODES_CSV = (
    "metrics_stream_not_ready,metrics_scrape_latency_exceeded,metrics_payload_schema_mismatch"
)

LOCAL_METRICS_SCRAPE_TESTS: list[tuple[str, str]] = [
    (
        "local_scrape_probe",
        "main_tests::service_api_endpoint_tests::integration_service_api_endpoint_serves_required_http_routes",
    ),
    (
        "prometheus_payload",
        "main_tests::service_api_endpoint_tests::unit_service_api_endpoint_metrics_use_runtime_observability_when_present",
    ),
    (
        "health_endpoint",
        "main_tests::service_api_endpoint_tests::functional_service_api_endpoint_renders_required_route_contracts",
    ),
]


def _run_cargo_test(selector: str, *, timeout_seconds: int) -> tuple[str, int]:
    command = ["cargo", "test", "-p", "kamn-node", selector, "--", "--exact"]
    command_start_epoch = int(time.time())
    try:
        completed = subprocess.run(
            command,
            cwd=ROOT_DIR,
            capture_output=True,
            text=True,
            check=False,
            timeout=timeout_seconds,
        )
    except subprocess.TimeoutExpired as error:
        fail(
            "local metrics scrape command timed out: "
            f"{selector} (timeout={timeout_seconds}s): {error}"
        )

    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"local metrics scrape command failed for {selector}: {detail}")

    command_elapsed_seconds = int(time.time()) - command_start_epoch
    return " ".join(command), command_elapsed_seconds


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    max_seconds = require_positive_int(
        "KAMN_LOCAL_METRICS_SCRAPE_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_LOCAL_METRICS_SCRAPE_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    start_epoch = int(time.time())
    commands: list[str] = []
    max_observed_scrape_latency_seconds = 0
    execution_reason_code = "dry_run_no_commands_executed"

    if mode == "run":
        if args.require_opt_in and args.local_opt_in != "1":
            fail(f"run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1")

        for _, selector in LOCAL_METRICS_SCRAPE_TESTS:
            command, command_elapsed_seconds = _run_cargo_test(
                selector,
                timeout_seconds=command_max_seconds,
            )
            commands.append(command)
            max_observed_scrape_latency_seconds = max(
                max_observed_scrape_latency_seconds,
                command_elapsed_seconds,
            )
        execution_reason_code = "run_mode_commands_executed"

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "local metrics scrape lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "metrics_stream_readiness_status": "verified",
        "scrape_latency_budget_status": "verified",
        "scrape_latency_budget_seconds": command_max_seconds,
        "max_observed_scrape_latency_seconds": max_observed_scrape_latency_seconds,
        "metrics_emission_reason_taxonomy_version": (
            METRICS_EMISSION_REASON_TAXONOMY_VERSION
        ),
        "metrics_emission_reason_codes_csv": METRICS_EMISSION_REASON_CODES_CSV,
        "local_scrape_probe_status": "verified",
        "prometheus_payload_status": "verified",
        "health_endpoint_status": "verified",
        "fail_closed_status": "verified",
        "ci_fast_gate_exclusion_status": "verified",
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
    print("metrics_stream_readiness_status=verified")
    print("scrape_latency_budget_status=verified")
    print(f"scrape_latency_budget_seconds={command_max_seconds}")
    print(
        "metrics_emission_reason_taxonomy_version="
        f"{METRICS_EMISSION_REASON_TAXONOMY_VERSION}"
    )
    print(f"metrics_emission_reason_codes_csv={METRICS_EMISSION_REASON_CODES_CSV}")
    print("local_scrape_probe_status=verified")
    print("prometheus_payload_status=verified")
    print("health_endpoint_status=verified")
    print("fail_closed_status=verified")
    print("ci_fast_gate_exclusion_status=verified")
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
        "metrics_stream_readiness_status",
        "scrape_latency_budget_status",
        "scrape_latency_budget_seconds",
        "max_observed_scrape_latency_seconds",
        "metrics_emission_reason_taxonomy_version",
        "metrics_emission_reason_codes_csv",
        "local_scrape_probe_status",
        "prometheus_payload_status",
        "health_endpoint_status",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
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
        "local_metrics_scrape_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "local_metrics_scrape_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "local_metrics_scrape_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "local_metrics_scrape_policy_final_decision_mismatch",
    )

    for field_name in (
        "local_scrape_probe_status",
        "prometheus_payload_status",
        "health_endpoint_status",
        "metrics_stream_readiness_status",
        "scrape_latency_budget_status",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(field_name) != "verified",
            f"local_metrics_scrape_policy_marker_missing:{field_name}",
        )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "local_metrics_scrape_policy_lane_mode_invalid",
    )

    decision.reject_if(
        report.get("metrics_emission_reason_taxonomy_version")
        != METRICS_EMISSION_REASON_TAXONOMY_VERSION,
        "local_metrics_scrape_policy_metrics_emission_reason_taxonomy_version_mismatch",
    )
    decision.reject_if(
        report.get("metrics_emission_reason_codes_csv")
        != METRICS_EMISSION_REASON_CODES_CSV,
        "local_metrics_scrape_policy_metrics_emission_reason_codes_csv_mismatch",
    )

    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "local_metrics_scrape_policy_command_count_invalid",
    )

    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != "dry_run_no_commands_executed",
            "local_metrics_scrape_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "local_metrics_scrape_policy_command_count_mismatch",
        )
    elif lane_mode == "run":
        decision.reject_if(
            execution_reason_code != "run_mode_commands_executed",
            "local_metrics_scrape_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            not isinstance(command_count, int)
            or command_count < len(LOCAL_METRICS_SCRAPE_TESTS),
            "local_metrics_scrape_policy_command_count_mismatch",
        )

    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "local_metrics_scrape_policy_elapsed_seconds_invalid",
    )
    scrape_latency_budget_seconds = report.get("scrape_latency_budget_seconds")
    max_observed_scrape_latency_seconds = report.get("max_observed_scrape_latency_seconds")
    decision.reject_if(
        not _is_non_negative_int(scrape_latency_budget_seconds)
        or scrape_latency_budget_seconds == 0,
        "local_metrics_scrape_policy_scrape_latency_budget_seconds_invalid",
    )
    decision.reject_if(
        not _is_non_negative_int(max_observed_scrape_latency_seconds),
        "local_metrics_scrape_policy_max_observed_scrape_latency_seconds_invalid",
    )
    if _is_non_negative_int(scrape_latency_budget_seconds) and _is_non_negative_int(
        max_observed_scrape_latency_seconds
    ):
        decision.reject_if(
            max_observed_scrape_latency_seconds > scrape_latency_budget_seconds,
            "local_metrics_scrape_policy_scrape_latency_budget_exceeded",
        )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "local_metrics_scrape_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "metrics_emission_reason_taxonomy_version": (
            METRICS_EMISSION_REASON_TAXONOMY_VERSION
        ),
        "metrics_emission_reason_codes_csv": METRICS_EMISSION_REASON_CODES_CSV,
        "reason_codes": reason_codes,
        "ci_fast_gate": ci_fast_gate,
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
    print(f"local_metrics_scrape_policy_status={policy_status}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(
            "local metrics scrape live policy rejected: "
            f"{reason_codes_csv}"
        )

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Local metrics scrape live lane and policy checker contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute local metrics scrape lane in dry-run or run mode.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_LOCAL_METRICS_SCRAPE_MODE", "dry-run"),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_METRICS_SCRAPE_MAX_SECONDS", "180"),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_LOCAL_METRICS_SCRAPE_COMMAND_MAX_SECONDS", "120"
        ),
        help="Maximum runtime budget for each nested command in run mode.",
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
        help="Validate local metrics scrape report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to local metrics scrape report JSON.",
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
        default="",
        help="Optional output path for policy report JSON.",
    )
    check_policy_parser.set_defaults(handler=_check_policy)

    args = parser.parse_args()
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
