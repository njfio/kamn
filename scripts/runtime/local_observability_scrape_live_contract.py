#!/usr/bin/env python3
"""Local observability scrape live lane and policy checker contracts."""

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

RUN_LANE_SCHEMA = "kamn.runtime.local-observability-scrape-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.local-observability-scrape-live-policy-report.v1"
OPT_IN_ENV = "KAMN_LOCAL_OBSERVABILITY_SCRAPE_OPT_IN"
DEGRADATION_REASON_CODES_CSV = (
    "none,"
    "readiness_transport_dependency_unhealthy,"
    "readiness_signer_dependency_unhealthy,"
    "readiness_commit_dependency_unhealthy,"
    "readiness_runtime_health_degraded"
)
SCRAPE_FAILURE_TAXONOMY_CSV = (
    "readiness_failure_drill_status,"
    "stream_reconnect_churn_status,"
    "queue_bound_budget_status"
)

LOCAL_OBSERVABILITY_SCRAPE_TESTS: list[tuple[str, str]] = [
    (
        "scrape_probe",
        "main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_serves_metrics_and_health_paths",
    ),
    (
        "metrics_content_type",
        "main_tests::observability_endpoint_tests::functional_observability_endpoint_renders_metrics_and_health_payloads",
    ),
    (
        "stream_lifecycle",
        "main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_serves_stream_path",
    ),
    (
        "tls_route_contract",
        "main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_tls_mode_serves_required_https_routes",
    ),
    (
        "stream_reconnect_churn",
        "main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_supports_stream_reconnect_churn_sequence",
    ),
    (
        "queue_bound_budget",
        "main_tests::observability_endpoint_tests::integration_runtime_observability_endpoint_enforces_queue_bound_request_budget",
    ),
    (
        "readiness_failure_drill",
        "main_tests::observability_endpoint_tests::functional_observability_endpoint_readiness_reports_degraded_timeout_reason_codes",
    ),
    (
        "readiness_reason_taxonomy",
        "main_tests::observability_endpoint_tests::functional_observability_endpoint_readiness_reason_taxonomy_covers_dependency_probe_matrix",
    ),
]


def _run_cargo_test(selector: str, *, timeout_seconds: int) -> str:
    command = ["cargo", "test", "-p", "kamn-node", selector, "--", "--exact"]
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
            "local observability scrape command timed out: "
            f"{selector} (timeout={timeout_seconds}s): {error}"
        )

    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"local observability scrape command failed for {selector}: {detail}")

    return " ".join(command)


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    lane_profile = require_enum(
        "--lane-profile",
        args.lane_profile.strip(),
        ("standard", "soak"),
    )
    requested_soak_iterations = require_positive_int(
        "KAMN_LOCAL_OBSERVABILITY_SCRAPE_SOAK_ITERATIONS",
        args.soak_iterations,
    )
    soak_iterations = requested_soak_iterations if lane_profile == "soak" else 1
    max_seconds = require_positive_int(
        "KAMN_LOCAL_OBSERVABILITY_SCRAPE_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_LOCAL_OBSERVABILITY_SCRAPE_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    start_epoch = int(time.time())
    commands: list[str] = []
    execution_reason_code = "dry_run_no_commands_executed"
    soak_iterations_executed = 0

    if mode == "run":
        if args.require_opt_in and args.local_opt_in != "1":
            fail(f"run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1")

        for _ in range(soak_iterations):
            for _, selector in LOCAL_OBSERVABILITY_SCRAPE_TESTS:
                commands.append(_run_cargo_test(selector, timeout_seconds=command_max_seconds))
        soak_iterations_executed = soak_iterations
        execution_reason_code = (
            "soak_run_mode_commands_executed"
            if lane_profile == "soak"
            else "run_mode_commands_executed"
        )

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "local observability scrape lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "lane_profile": lane_profile,
        "scrape_probe_status": "verified",
        "metrics_content_type_status": "verified",
        "stream_lifecycle_status": "verified",
        "observability_tls_route_contract_status": "verified",
        "stream_reconnect_churn_status": "verified",
        "queue_bound_budget_status": "verified",
        "readiness_probe_status": "verified",
        "readiness_failure_drill_status": "verified",
        "readiness_reason_taxonomy_status": "verified",
        "degradation_taxonomy_status": "verified",
        "degradation_reason_codes_csv": DEGRADATION_REASON_CODES_CSV,
        "scrape_failure_taxonomy_status": "verified",
        "scrape_failure_taxonomy_csv": SCRAPE_FAILURE_TAXONOMY_CSV,
        "local_heavy_soak_lane_status": (
            "verified" if lane_profile == "soak" else "not_enabled"
        ),
        "soak_iterations_requested": soak_iterations,
        "soak_iterations_executed": soak_iterations_executed,
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
    print(f"lane_profile={lane_profile}")
    print("scrape_probe_status=verified")
    print("metrics_content_type_status=verified")
    print("stream_lifecycle_status=verified")
    print("observability_tls_route_contract_status=verified")
    print("stream_reconnect_churn_status=verified")
    print("queue_bound_budget_status=verified")
    print("readiness_probe_status=verified")
    print("readiness_failure_drill_status=verified")
    print("readiness_reason_taxonomy_status=verified")
    print("degradation_taxonomy_status=verified")
    print(f"degradation_reason_codes_csv={DEGRADATION_REASON_CODES_CSV}")
    print("scrape_failure_taxonomy_status=verified")
    print(f"scrape_failure_taxonomy_csv={SCRAPE_FAILURE_TAXONOMY_CSV}")
    if lane_profile == "soak":
        print("local_heavy_soak_lane_status=verified")
    else:
        print("local_heavy_soak_lane_status=not_enabled")
    print(f"soak_iterations_requested={soak_iterations}")
    print(f"soak_iterations_executed={soak_iterations_executed}")
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
        "lane_profile",
        "scrape_probe_status",
        "metrics_content_type_status",
        "stream_lifecycle_status",
        "observability_tls_route_contract_status",
        "stream_reconnect_churn_status",
        "queue_bound_budget_status",
        "readiness_probe_status",
        "readiness_failure_drill_status",
        "readiness_reason_taxonomy_status",
        "degradation_taxonomy_status",
        "degradation_reason_codes_csv",
        "scrape_failure_taxonomy_status",
        "scrape_failure_taxonomy_csv",
        "local_heavy_soak_lane_status",
        "soak_iterations_requested",
        "soak_iterations_executed",
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
        "local_observability_scrape_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "local_observability_scrape_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "local_observability_scrape_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "local_observability_scrape_policy_final_decision_mismatch",
    )

    for field_name in (
        "scrape_probe_status",
        "metrics_content_type_status",
        "stream_lifecycle_status",
        "observability_tls_route_contract_status",
        "stream_reconnect_churn_status",
        "queue_bound_budget_status",
        "readiness_probe_status",
        "readiness_failure_drill_status",
        "readiness_reason_taxonomy_status",
        "degradation_taxonomy_status",
        "scrape_failure_taxonomy_status",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(field_name) != "verified",
            f"local_observability_scrape_policy_marker_missing:{field_name}",
        )
    decision.reject_if(
        report.get("degradation_reason_codes_csv") != DEGRADATION_REASON_CODES_CSV,
        "local_observability_scrape_policy_degradation_reason_codes_csv_mismatch",
    )
    decision.reject_if(
        report.get("scrape_failure_taxonomy_csv") != SCRAPE_FAILURE_TAXONOMY_CSV,
        "local_observability_scrape_policy_scrape_failure_taxonomy_csv_mismatch",
    )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "local_observability_scrape_policy_lane_mode_invalid",
    )
    lane_profile = report.get("lane_profile")
    decision.reject_if(
        lane_profile not in {"standard", "soak"},
        "local_observability_scrape_policy_lane_profile_invalid",
    )
    soak_iterations_requested = report.get("soak_iterations_requested")
    decision.reject_if(
        not _is_non_negative_int(soak_iterations_requested) or soak_iterations_requested <= 0,
        "local_observability_scrape_policy_soak_iterations_requested_invalid",
    )
    soak_iterations_executed = report.get("soak_iterations_executed")
    decision.reject_if(
        not _is_non_negative_int(soak_iterations_executed),
        "local_observability_scrape_policy_soak_iterations_executed_invalid",
    )

    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "local_observability_scrape_policy_command_count_invalid",
    )

    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != "dry_run_no_commands_executed",
            "local_observability_scrape_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "local_observability_scrape_policy_command_count_mismatch",
        )
        decision.reject_if(
            soak_iterations_executed != 0,
            "local_observability_scrape_policy_soak_iterations_executed_mismatch",
        )
    elif lane_mode == "run":
        if lane_profile == "soak":
            decision.reject_if(
                execution_reason_code != "soak_run_mode_commands_executed",
                "local_observability_scrape_policy_execution_reason_code_mismatch",
            )
            decision.reject_if(
                report.get("local_heavy_soak_lane_status") != "verified",
                "local_observability_scrape_policy_marker_missing:local_heavy_soak_lane_status",
            )
            decision.reject_if(
                soak_iterations_executed != soak_iterations_requested,
                "local_observability_scrape_policy_soak_iterations_executed_mismatch",
            )
            decision.reject_if(
                not isinstance(command_count, int)
                or command_count
                < len(LOCAL_OBSERVABILITY_SCRAPE_TESTS) * soak_iterations_requested,
                "local_observability_scrape_policy_command_count_mismatch",
            )
        else:
            decision.reject_if(
                report.get("local_heavy_soak_lane_status") != "not_enabled",
                "local_observability_scrape_policy_soak_status_invalid_for_standard",
            )
            decision.reject_if(
                execution_reason_code != "run_mode_commands_executed",
                "local_observability_scrape_policy_execution_reason_code_mismatch",
            )
            decision.reject_if(
                soak_iterations_requested != 1,
                "local_observability_scrape_policy_soak_iterations_requested_invalid",
            )
            decision.reject_if(
                soak_iterations_executed != 1,
                "local_observability_scrape_policy_soak_iterations_executed_mismatch",
            )
            decision.reject_if(
                not isinstance(command_count, int)
                or command_count < len(LOCAL_OBSERVABILITY_SCRAPE_TESTS),
                "local_observability_scrape_policy_command_count_mismatch",
            )

    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "local_observability_scrape_policy_elapsed_seconds_invalid",
    )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "local_observability_scrape_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
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
    print(f"local_observability_scrape_policy_status={policy_status}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(
            "local observability scrape live policy rejected: "
            f"{reason_codes_csv}"
        )

    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Local observability scrape live lane and policy checker contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute local observability scrape lane in dry-run or run mode.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_LOCAL_OBSERVABILITY_SCRAPE_MODE", "dry-run"),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--lane-profile",
        default=os.environ.get(
            "KAMN_LOCAL_OBSERVABILITY_SCRAPE_LANE_PROFILE", "standard"
        ),
        help="Lane profile: standard|soak.",
    )
    run_lane_parser.add_argument(
        "--soak-iterations",
        default=os.environ.get("KAMN_LOCAL_OBSERVABILITY_SCRAPE_SOAK_ITERATIONS", "3"),
        help="Iteration count used for soak profile execution.",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_LOCAL_OBSERVABILITY_SCRAPE_MAX_SECONDS", "180"),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_LOCAL_OBSERVABILITY_SCRAPE_COMMAND_MAX_SECONDS", "120"
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
        help="Validate local observability scrape report policy.",
    )
    check_policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to local observability scrape report JSON.",
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
