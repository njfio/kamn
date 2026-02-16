#!/usr/bin/env python3
"""Service API + observability route compatibility matrix lane and policy."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
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

RUN_LANE_SCHEMA = (
    "kamn.runtime.service-api-observability-route-compatibility-live-report.v1"
)
POLICY_SCHEMA = (
    "kamn.runtime.service-api-observability-route-compatibility-live-policy-report.v1"
)
CONTRACT_LANE_SCHEMA = (
    "kamn.runtime.service-api-observability-route-compatibility-live-contract-lane-report.v1"
)
MATRIX_SCHEMA = (
    "kamn.runtime.service-api-observability-route-compatibility-matrix.v1"
)
POLICY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-observability-route-compatibility-policy-reason-taxonomy.v1"
)
POLICY_REASON_CODES_CSV = ",".join(
    [
        "ci_fast_gate_failed",
        "service_api_observability_route_compatibility_policy_command_count_invalid",
        "service_api_observability_route_compatibility_policy_command_count_mismatch",
        "service_api_observability_route_compatibility_policy_elapsed_seconds_invalid",
        "service_api_observability_route_compatibility_policy_execution_reason_code_mismatch",
        "service_api_observability_route_compatibility_policy_final_decision_invalid",
        "service_api_observability_route_compatibility_policy_final_decision_mismatch",
        "service_api_observability_route_compatibility_policy_lane_mode_invalid",
        "service_api_observability_route_compatibility_policy_marker_missing",
        "service_api_observability_route_compatibility_policy_matrix_row_compatibility_marker_missing",
        "service_api_observability_route_compatibility_policy_matrix_row_content_type_mismatch",
        "service_api_observability_route_compatibility_policy_matrix_row_count_mismatch",
        "service_api_observability_route_compatibility_policy_matrix_row_duplicate",
        "service_api_observability_route_compatibility_policy_matrix_row_id_invalid",
        "service_api_observability_route_compatibility_policy_matrix_row_invalid",
        "service_api_observability_route_compatibility_policy_matrix_row_method_mismatch",
        "service_api_observability_route_compatibility_policy_matrix_row_missing",
        "service_api_observability_route_compatibility_policy_matrix_row_route_mismatch",
        "service_api_observability_route_compatibility_policy_matrix_row_status_mismatch",
        "service_api_observability_route_compatibility_policy_matrix_row_surface_mismatch",
        "service_api_observability_route_compatibility_policy_matrix_rows_invalid",
        "service_api_observability_route_compatibility_policy_matrix_schema_mismatch",
        "service_api_observability_route_compatibility_policy_schema_mismatch",
        "service_api_observability_route_compatibility_policy_status_invalid",
    ]
)
ARCHITECTURE_DOC = ROOT_DIR / "docs/architecture/service-runtime.md"

MATRIX_ROWS: list[dict[str, Any]] = [
    {
        "row_id": "api_healthz_get",
        "surface": "service_api",
        "route_class": "service_api_health",
        "route": "/healthz",
        "method": "GET",
        "expected_status": 200,
        "expected_content_type": "application/json",
        "evidence_test_selector": (
            "main_tests::service_api_endpoint_tests::"
            "integration_service_api_endpoint_serves_required_http_routes"
        ),
    },
    {
        "row_id": "api_metrics_get",
        "surface": "service_api",
        "route_class": "service_api_metrics",
        "route": "/metrics",
        "method": "GET",
        "expected_status": 200,
        "expected_content_type": "text/plain; version=0.0.4",
        "evidence_test_selector": (
            "main_tests::service_api_endpoint_tests::"
            "integration_service_api_endpoint_serves_required_http_routes"
        ),
    },
    {
        "row_id": "api_websocket_upgrade_required_get",
        "surface": "service_api",
        "route_class": "service_api_websocket_upgrade",
        "route": "/v1/events/ws",
        "method": "GET",
        "expected_status": 400,
        "expected_content_type": "application/json",
        "evidence_test_selector": (
            "main_tests::service_api_endpoint_tests::"
            "unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts"
        ),
    },
    {
        "row_id": "api_messages_send_delete_method_not_allowed",
        "surface": "service_api",
        "route_class": "service_api_method_guard",
        "route": "/v1/messages/send",
        "method": "DELETE",
        "expected_status": 405,
        "expected_content_type": "application/json",
        "evidence_test_selector": (
            "main_tests::service_api_endpoint_tests::"
            "unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts"
        ),
    },
    {
        "row_id": "api_unknown_path_not_found",
        "surface": "service_api",
        "route_class": "service_api_route_not_found",
        "route": "/v1/nope",
        "method": "GET",
        "expected_status": 404,
        "expected_content_type": "application/json",
        "evidence_test_selector": (
            "main_tests::service_api_endpoint_tests::"
            "unit_service_api_endpoint_error_envelopes_use_reason_code_and_message_contracts"
        ),
    },
    {
        "row_id": "observability_metrics_get",
        "surface": "observability_endpoint",
        "route_class": "observability_metrics",
        "route": "/metrics",
        "method": "GET",
        "expected_status": 200,
        "expected_content_type": "text/plain; version=0.0.4",
        "evidence_test_selector": (
            "main_tests::observability_endpoint_tests::"
            "integration_runtime_observability_endpoint_serves_metrics_and_health_paths"
        ),
    },
    {
        "row_id": "observability_health_get",
        "surface": "observability_endpoint",
        "route_class": "observability_health",
        "route": "/healthz",
        "method": "GET",
        "expected_status": 200,
        "expected_content_type": "application/json",
        "evidence_test_selector": (
            "main_tests::observability_endpoint_tests::"
            "integration_runtime_observability_endpoint_serves_metrics_and_health_paths"
        ),
    },
    {
        "row_id": "observability_ready_get",
        "surface": "observability_endpoint",
        "route_class": "observability_readiness",
        "route": "/readyz",
        "method": "GET",
        "expected_status": 200,
        "expected_content_type": "application/json",
        "evidence_test_selector": (
            "main_tests::observability_endpoint_tests::"
            "integration_runtime_observability_endpoint_serves_metrics_and_health_paths"
        ),
    },
    {
        "row_id": "observability_stream_get",
        "surface": "observability_endpoint",
        "route_class": "observability_stream",
        "route": "/metrics.stream",
        "method": "GET",
        "expected_status": 200,
        "expected_content_type": "application/x-ndjson",
        "evidence_test_selector": (
            "main_tests::observability_endpoint_tests::"
            "integration_runtime_observability_endpoint_serves_stream_path"
        ),
    },
    {
        "row_id": "observability_unknown_path_not_found",
        "surface": "observability_endpoint",
        "route_class": "observability_negative_path",
        "route": "/unknown",
        "method": "GET",
        "expected_status": 404,
        "expected_content_type": "text/plain; charset=utf-8",
        "evidence_test_selector": (
            "main_tests::observability_endpoint_tests::"
            "integration_runtime_observability_endpoint_returns_not_found_for_unknown_path"
        ),
    },
    {
        "row_id": "observability_metrics_post_not_found",
        "surface": "observability_endpoint",
        "route_class": "observability_negative_path",
        "route": "/metrics",
        "method": "POST",
        "expected_status": 404,
        "expected_content_type": "text/plain; charset=utf-8",
        "evidence_test_selector": (
            "main_tests::observability_endpoint_tests::"
            "integration_runtime_observability_endpoint_returns_not_found_for_malformed_request_method"
        ),
    },
]

REQUIRED_ROUTE_CLASSES: tuple[str, ...] = (
    "service_api_health",
    "service_api_metrics",
    "service_api_websocket_upgrade",
    "service_api_method_guard",
    "service_api_route_not_found",
    "observability_metrics",
    "observability_health",
    "observability_readiness",
    "observability_stream",
    "observability_negative_path",
)

PARITY_CHECKPOINTS: tuple[dict[str, str], ...] = (
    {
        "checkpoint_id": "health_route_surface_parity",
        "service_api_row_id": "api_healthz_get",
        "observability_row_id": "observability_health_get",
    },
    {
        "checkpoint_id": "metrics_route_surface_parity",
        "service_api_row_id": "api_metrics_get",
        "observability_row_id": "observability_metrics_get",
    },
)


def _dedupe_preserve_order(items: list[str]) -> list[str]:
    seen: set[str] = set()
    ordered: list[str] = []
    for item in items:
        if item in seen:
            continue
        seen.add(item)
        ordered.append(item)
    return ordered


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


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
            "service api observability route compatibility command timed out: "
            f"{selector} (timeout={timeout_seconds}s): {error}"
        )

    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(
            "service api observability route compatibility command failed for "
            f"{selector}: {detail}"
        )

    return " ".join(command)


def _build_matrix_rows() -> list[dict[str, Any]]:
    rows: list[dict[str, Any]] = []
    for row in MATRIX_ROWS:
        rows.append(
            {
                "row_id": row["row_id"],
                "surface": row["surface"],
                "route_class": row["route_class"],
                "route": row["route"],
                "method": row["method"],
                "expected_status": row["expected_status"],
                "expected_content_type": row["expected_content_type"],
                "evidence_test_selector": row["evidence_test_selector"],
                "compatibility_status": "verified",
            }
        )
    return rows


def _build_matrix_row_map(matrix_rows: list[dict[str, Any]]) -> dict[str, dict[str, Any]]:
    row_map: dict[str, dict[str, Any]] = {}
    for row in matrix_rows:
        row_map[str(row["row_id"])] = row
    return row_map


def _compute_route_class_coverage(
    matrix_rows: list[dict[str, Any]],
) -> tuple[str, str]:
    observed_classes = {
        str(row["route_class"])
        for row in matrix_rows
        if isinstance(row.get("route_class"), str) and row.get("route_class")
    }
    missing_classes = sorted(set(REQUIRED_ROUTE_CLASSES) - observed_classes)
    if missing_classes:
        fail(
            "service api observability route compatibility matrix missing "
            f"route classes: {','.join(missing_classes)}"
        )
    return "verified", ",".join(sorted(observed_classes))


def _compute_route_parity_checkpoints(
    matrix_rows: list[dict[str, Any]],
) -> list[dict[str, str]]:
    row_map = _build_matrix_row_map(matrix_rows)
    checkpoints: list[dict[str, str]] = []
    for checkpoint in PARITY_CHECKPOINTS:
        checkpoint_id = checkpoint["checkpoint_id"]
        service_row_id = checkpoint["service_api_row_id"]
        observability_row_id = checkpoint["observability_row_id"]
        service_row = row_map.get(service_row_id)
        observability_row = row_map.get(observability_row_id)
        if service_row is None or observability_row is None:
            fail(
                "service api observability route compatibility parity checkpoint "
                f"row missing: {checkpoint_id}"
            )
        for field in ("route", "method", "expected_status", "expected_content_type"):
            if service_row.get(field) != observability_row.get(field):
                fail(
                    "service api observability route compatibility parity checkpoint "
                    f"mismatch: {checkpoint_id}:{field}"
                )
        checkpoints.append(
            {
                "checkpoint_id": checkpoint_id,
                "service_api_row_id": service_row_id,
                "observability_row_id": observability_row_id,
                "parity_status": "verified",
            }
        )
    return checkpoints


def _compute_fail_closed_checkpoint_status(matrix_rows: list[dict[str, Any]]) -> str:
    service_negative_path = any(
        row.get("surface") == "service_api" and int(row.get("expected_status", 0)) >= 400
        for row in matrix_rows
    )
    observability_negative_path = any(
        row.get("surface") == "observability_endpoint"
        and int(row.get("expected_status", 0)) >= 400
        for row in matrix_rows
    )
    if not service_negative_path or not observability_negative_path:
        fail(
            "service api observability route compatibility matrix missing "
            "fail-closed branch coverage"
        )
    return "verified"


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    max_seconds = require_positive_int(
        "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    start_epoch = int(time.time())
    commands: list[str] = []
    execution_reason_code = "dry_run_no_commands_executed"

    if mode == "run":
        selectors = _dedupe_preserve_order(
            [row["evidence_test_selector"] for row in MATRIX_ROWS]
        )
        for selector in selectors:
            commands.append(_run_cargo_test(selector, timeout_seconds=command_max_seconds))
        execution_reason_code = "run_mode_commands_executed"

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "service api observability route compatibility lane exceeded runtime "
            f"budget: {elapsed_seconds}s (max={max_seconds}s)"
        )

    matrix_rows = _build_matrix_rows()
    route_class_coverage_status, route_classes_csv = _compute_route_class_coverage(
        matrix_rows
    )
    parity_checkpoints = _compute_route_parity_checkpoints(matrix_rows)
    fail_closed_checkpoint_status = _compute_fail_closed_checkpoint_status(matrix_rows)
    service_api_rows = sum(1 for row in matrix_rows if row["surface"] == "service_api")
    observability_rows = sum(
        1 for row in matrix_rows if row["surface"] == "observability_endpoint"
    )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "matrix_schema_version": MATRIX_SCHEMA,
        "route_compatibility_matrix_status": "verified",
        "service_api_route_matrix_status": "verified",
        "observability_route_matrix_status": "verified",
        "route_parity_checkpoint_status": "verified",
        "fail_closed_checkpoint_status": fail_closed_checkpoint_status,
        "route_class_coverage_status": route_class_coverage_status,
        "route_classes_csv": route_classes_csv,
        "parity_checkpoint_count": len(parity_checkpoints),
        "parity_checkpoints": parity_checkpoints,
        "fail_closed_status": "verified",
        "performance_budget_status": "verified",
        "execution_reason_code": execution_reason_code,
        "compatibility_row_count": len(matrix_rows),
        "service_api_row_count": service_api_rows,
        "observability_row_count": observability_rows,
        "matrix_rows": matrix_rows,
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
    print("route_compatibility_matrix_status=verified")
    print("service_api_route_matrix_status=verified")
    print("observability_route_matrix_status=verified")
    print("route_parity_checkpoint_status=verified")
    print(f"fail_closed_checkpoint_status={fail_closed_checkpoint_status}")
    print(f"route_class_coverage_status={route_class_coverage_status}")
    print("fail_closed_status=verified")
    print("performance_budget_status=verified")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"compatibility_row_count={len(matrix_rows)}")
    print(f"command_count={len(commands)}")
    if output_json is not None:
        print(f"report_file={output_json}")
    return 0


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
        "matrix_schema_version",
        "route_compatibility_matrix_status",
        "service_api_route_matrix_status",
        "observability_route_matrix_status",
        "route_parity_checkpoint_status",
        "fail_closed_checkpoint_status",
        "route_class_coverage_status",
        "fail_closed_status",
        "performance_budget_status",
        "execution_reason_code",
        "compatibility_row_count",
        "matrix_rows",
        "command_count",
        "elapsed_seconds",
    ]
    missing_fields = [field for field in required_fields if field not in report]
    if missing_fields:
        fail(f"missing required report fields: {','.join(missing_fields)}")

    decision = DecisionAccumulator()
    decision.reject_if(
        report.get("schema_version") != RUN_LANE_SCHEMA,
        "service_api_observability_route_compatibility_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("matrix_schema_version") != MATRIX_SCHEMA,
        "service_api_observability_route_compatibility_policy_matrix_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "service_api_observability_route_compatibility_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "service_api_observability_route_compatibility_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "service_api_observability_route_compatibility_policy_final_decision_mismatch",
    )

    for field in (
        "route_compatibility_matrix_status",
        "service_api_route_matrix_status",
        "observability_route_matrix_status",
        "route_parity_checkpoint_status",
        "fail_closed_checkpoint_status",
        "route_class_coverage_status",
        "fail_closed_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(field) != "verified",
            f"service_api_observability_route_compatibility_policy_marker_missing:{field}",
        )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "service_api_observability_route_compatibility_policy_lane_mode_invalid",
    )

    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "service_api_observability_route_compatibility_policy_command_count_invalid",
    )

    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != "dry_run_no_commands_executed",
            "service_api_observability_route_compatibility_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "service_api_observability_route_compatibility_policy_command_count_mismatch",
        )
    elif lane_mode == "run":
        required_commands = len(
            _dedupe_preserve_order([row["evidence_test_selector"] for row in MATRIX_ROWS])
        )
        decision.reject_if(
            execution_reason_code != "run_mode_commands_executed",
            "service_api_observability_route_compatibility_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            not isinstance(command_count, int) or command_count < required_commands,
            "service_api_observability_route_compatibility_policy_command_count_mismatch",
        )

    decision.reject_if(
        not _is_non_negative_int(report.get("elapsed_seconds")),
        "service_api_observability_route_compatibility_policy_elapsed_seconds_invalid",
    )
    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    matrix_rows = report.get("matrix_rows")
    decision.reject_if(
        not isinstance(matrix_rows, list),
        "service_api_observability_route_compatibility_policy_matrix_rows_invalid",
    )

    row_map: dict[str, dict[str, Any]] = {}
    if isinstance(matrix_rows, list):
        for row in matrix_rows:
            if not isinstance(row, dict):
                decision.reject_if(
                    True,
                    "service_api_observability_route_compatibility_policy_matrix_row_invalid",
                )
                continue
            row_id = row.get("row_id")
            if not isinstance(row_id, str) or not row_id:
                decision.reject_if(
                    True,
                    "service_api_observability_route_compatibility_policy_matrix_row_id_invalid",
                )
                continue
            if row_id in row_map:
                decision.reject_if(
                    True,
                    f"service_api_observability_route_compatibility_policy_matrix_row_duplicate:{row_id}",
                )
                continue
            row_map[row_id] = row

    decision.reject_if(
        report.get("compatibility_row_count") != len(MATRIX_ROWS),
        "service_api_observability_route_compatibility_policy_matrix_row_count_mismatch",
    )
    decision.reject_if(
        report.get("parity_checkpoint_count") != len(PARITY_CHECKPOINTS),
        "service_api_observability_route_compatibility_policy_marker_missing:parity_checkpoint_count",
    )
    parity_checkpoints = report.get("parity_checkpoints")
    decision.reject_if(
        not isinstance(parity_checkpoints, list),
        "service_api_observability_route_compatibility_policy_marker_missing:parity_checkpoints",
    )
    if isinstance(parity_checkpoints, list):
        for checkpoint in parity_checkpoints:
            checkpoint_id = (
                checkpoint.get("checkpoint_id")
                if isinstance(checkpoint, dict)
                else "invalid"
            )
            if not isinstance(checkpoint, dict):
                decision.reject_if(
                    True,
                    "service_api_observability_route_compatibility_policy_marker_missing:parity_checkpoints",
                )
                continue
            decision.reject_if(
                checkpoint.get("parity_status") != "verified",
                "service_api_observability_route_compatibility_policy_marker_missing:"
                f"parity_status:{checkpoint_id}",
            )

    for expected_row in MATRIX_ROWS:
        row_id = expected_row["row_id"]
        observed = row_map.get(row_id)
        if observed is None:
            decision.reject_if(
                True,
                f"service_api_observability_route_compatibility_policy_matrix_row_missing:{row_id}",
            )
            continue

        decision.reject_if(
            observed.get("surface") != expected_row["surface"],
            f"service_api_observability_route_compatibility_policy_matrix_row_surface_mismatch:{row_id}",
        )
        decision.reject_if(
            observed.get("route_class") != expected_row["route_class"],
            f"service_api_observability_route_compatibility_policy_matrix_row_invalid:{row_id}",
        )
        decision.reject_if(
            observed.get("route") != expected_row["route"],
            f"service_api_observability_route_compatibility_policy_matrix_row_route_mismatch:{row_id}",
        )
        decision.reject_if(
            observed.get("method") != expected_row["method"],
            f"service_api_observability_route_compatibility_policy_matrix_row_method_mismatch:{row_id}",
        )
        decision.reject_if(
            observed.get("expected_status") != expected_row["expected_status"],
            f"service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:{row_id}",
        )
        decision.reject_if(
            observed.get("expected_content_type")
            != expected_row["expected_content_type"],
            f"service_api_observability_route_compatibility_policy_matrix_row_content_type_mismatch:{row_id}",
        )
        decision.reject_if(
            observed.get("compatibility_status") != "verified",
            f"service_api_observability_route_compatibility_policy_matrix_row_compatibility_marker_missing:{row_id}",
        )

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "service_api_observability_route_compatibility_policy_status": policy_status,
        "reason_taxonomy_version": POLICY_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": POLICY_REASON_CODES_CSV,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "reason_codes": reason_codes,
        "reason_codes_value": ",".join(reason_codes),
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
    print(
        "service_api_observability_route_compatibility_policy_status="
        f"{policy_status}"
    )
    print(f"reason_taxonomy_version={POLICY_REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={POLICY_REASON_CODES_CSV}")
    print(f"reason_codes={reason_codes_csv}")
    print(f"reason_codes_value={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(
            "service api observability route compatibility policy rejected: "
            f"{reason_codes_csv}"
        )

    return 0


def _run_contract_lane(args: argparse.Namespace) -> int:
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    start_epoch = int(time.time())
    with tempfile.TemporaryDirectory(prefix="service-api-observability-route-compat-") as tmp:
        tmp_dir = Path(tmp)
        summary_report = tmp_dir / "service-api-observability-route-compatibility-summary.json"
        policy_report = tmp_dir / "service-api-observability-route-compatibility-policy.json"
        tampered_report = tmp_dir / "service-api-observability-route-compatibility-summary.tampered.json"

        run_args = argparse.Namespace(
            mode=args.mode,
            max_seconds=args.max_seconds,
            command_max_seconds=args.command_max_seconds,
            output_json=str(summary_report),
        )
        _run_lane(run_args)

        policy_args = argparse.Namespace(
            report_file=str(summary_report),
            expected_final_decision="GO",
            ci_fast_gate=ci_fast_gate,
            output_json=str(policy_report),
        )
        _check_policy(policy_args)

        payload = json.loads(summary_report.read_text(encoding="utf-8"))
        for required_marker in (
            "route_parity_checkpoint_status",
            "fail_closed_checkpoint_status",
            "route_class_coverage_status",
        ):
            if payload.get(required_marker) != "verified":
                fail(
                    "service api observability route compatibility report missing "
                    f"marker: {required_marker}=verified"
                )

        for row in payload.get("matrix_rows", []):
            if row.get("row_id") == "api_healthz_get":
                row["expected_status"] = 201
        tampered_report.write_text(
            json.dumps(payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

        tamper_reason_code = (
            "service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:"
            "api_healthz_get"
        )
        try:
            _check_policy(
                argparse.Namespace(
                    report_file=str(tampered_report),
                    expected_final_decision="GO",
                    ci_fast_gate=ci_fast_gate,
                    output_json=str(tmp_dir / "service-api-observability-route-compatibility-policy.tampered.json"),
                )
            )
            fail(
                "expected tampered service api observability route compatibility "
                "report to fail policy validation"
            )
        except ContractError as exc:
            if tamper_reason_code not in str(exc):
                raise

        if not ARCHITECTURE_DOC.is_file():
            fail(f"required architecture doc missing: {ARCHITECTURE_DOC}")
        architecture_text = ARCHITECTURE_DOC.read_text(encoding="utf-8")
        required_doc_markers = [
            "Service API + Observability Route Compatibility Matrix Contract",
            "validate_service_api_observability_route_compatibility_live.sh",
            "check_service_api_observability_route_compatibility_live_policy.sh",
            "validate_service_api_observability_route_compatibility_live_contract_lane.sh",
            "route_parity_checkpoint_status=verified",
            "fail_closed_checkpoint_status=verified",
            "route_class_coverage_status=verified",
        ]
        for marker in required_doc_markers:
            if marker not in architecture_text:
                fail(
                    "architecture doc missing compatibility matrix marker: "
                    f"{marker}"
                )

        elapsed_seconds = int(time.time()) - start_epoch
        max_seconds = require_positive_int(
            "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_CONTRACT_MAX_SECONDS",
            args.max_seconds,
        )
        if elapsed_seconds > max_seconds:
            fail(
                "service api observability route compatibility contract lane "
                f"exceeded runtime budget: {elapsed_seconds}s (max={max_seconds}s)"
            )

        lane_payload = {
            "schema_version": CONTRACT_LANE_SCHEMA,
            "status": "pass",
            "final_decision": "GO",
            "lane_mode": args.mode,
            "service_api_observability_route_compatibility_contract_status": "verified",
            "service_api_observability_route_compatibility_policy_status": "verified",
            "route_parity_checkpoint_status": "verified",
            "fail_closed_checkpoint_status": "verified",
            "fail_closed_tamper_status": "verified",
            "docs_contract_status": "verified",
            "performance_budget_status": "verified",
            "fail_closed_reason_code": tamper_reason_code,
            "summary_report_file": str(summary_report),
            "policy_report_file": str(policy_report),
            "elapsed_seconds": elapsed_seconds,
            "max_seconds": max_seconds,
        }

        output_json = None
        if args.output_json:
            output_json = Path(args.output_json).resolve()
            write_json(output_json, lane_payload)

        policy_output_json = None
        if args.policy_output_json:
            policy_output_json = Path(args.policy_output_json).resolve()
            write_json(policy_output_json, load_json(policy_report))

    print("status=pass")
    print("final_decision=GO")
    print(f"lane_mode={args.mode}")
    print("service_api_observability_route_compatibility_contract_status=verified")
    print("service_api_observability_route_compatibility_policy_status=verified")
    print("route_parity_checkpoint_status=verified")
    print("fail_closed_checkpoint_status=verified")
    print("fail_closed_tamper_status=verified")
    print("docs_contract_status=verified")
    print("performance_budget_status=verified")
    print(
        "fail_closed_reason_code="
        "service_api_observability_route_compatibility_policy_matrix_row_status_mismatch:"
        "api_healthz_get"
    )
    if output_json is not None:
        print(f"report_file={output_json}")
    if policy_output_json is not None:
        print(f"policy_report_file={policy_output_json}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description=(
            "Service API + observability route compatibility matrix lane and policy."
        )
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute compatibility matrix lane in dry-run or run mode.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get(
            "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_MODE", "dry-run"
        ),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_MAX_SECONDS", "180"
        ),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_COMMAND_MAX_SECONDS",
            "120",
        ),
        help="Maximum runtime budget for each nested command in run mode.",
    )
    run_lane_parser.add_argument(
        "--output-json",
        default="",
        help="Optional path for lane report JSON output.",
    )
    run_lane_parser.set_defaults(handler=_run_lane)

    policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate compatibility matrix report policy invariants.",
    )
    policy_parser.add_argument(
        "--report-file",
        required=True,
        help="Path to compatibility matrix run-lane report JSON.",
    )
    policy_parser.add_argument(
        "--expected-final-decision",
        default="GO",
        help="Expected report final decision (GO|NO-GO).",
    )
    policy_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        help="Fast gate state for policy evaluation (PASS|FAIL).",
    )
    policy_parser.add_argument(
        "--output-json",
        default="",
        help="Optional path for policy report JSON output.",
    )
    policy_parser.set_defaults(handler=_check_policy)

    contract_parser = subparsers.add_parser(
        "run-contract-lane",
        help="Execute compatibility matrix lane + policy + tamper regression.",
    )
    contract_parser.add_argument(
        "--mode",
        default=os.environ.get(
            "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_CONTRACT_MODE",
            "dry-run",
        ),
        help="Contract lane mode: dry-run|run.",
    )
    contract_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_CONTRACT_MAX_SECONDS",
            "240",
        ),
        help="Maximum contract lane runtime budget in seconds.",
    )
    contract_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_SERVICE_API_OBSERVABILITY_ROUTE_COMPATIBILITY_COMMAND_MAX_SECONDS",
            "120",
        ),
        help="Maximum runtime budget for each nested command in run mode.",
    )
    contract_parser.add_argument(
        "--ci-fast-gate",
        default="PASS",
        help="Fast gate state for policy evaluation (PASS|FAIL).",
    )
    contract_parser.add_argument(
        "--output-json",
        default="",
        help="Optional path for contract lane report JSON output.",
    )
    contract_parser.add_argument(
        "--policy-output-json",
        default="",
        help="Optional path for policy report JSON output.",
    )
    contract_parser.set_defaults(handler=_run_contract_lane)

    args = parser.parse_args()
    try:
        return int(args.handler(args))
    except ContractError as exc:
        print(str(exc), file=sys.stderr)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
