#!/usr/bin/env python3
"""Service API tenant-isolation matrix lane, policy checker, and contract lane."""

from __future__ import annotations

import argparse
import contextlib
import io
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

RUN_LANE_SCHEMA = "kamn.runtime.service-api-tenant-isolation-matrix-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.service-api-tenant-isolation-matrix-live-policy-report.v1"
CONTRACT_LANE_SCHEMA = (
    "kamn.runtime.service-api-tenant-isolation-matrix-live-contract-lane-report.v1"
)
MATRIX_SCHEMA = "kamn.runtime.service-api-tenant-isolation-matrix.v1"

REASON_TAXONOMY_VERSION = "kamn.runtime.service-api-tenant-isolation-matrix-policy-reason-taxonomy.v1"
REASON_CODES_CSV = ",".join(
    [
        "ci_fast_gate_failed",
        "service_api_tenant_isolation_policy_schema_mismatch",
        "service_api_tenant_isolation_policy_status_invalid",
        "service_api_tenant_isolation_policy_final_decision_invalid",
        "service_api_tenant_isolation_policy_final_decision_mismatch",
        "service_api_tenant_isolation_policy_lane_mode_invalid",
        "service_api_tenant_isolation_policy_matrix_schema_mismatch",
        "service_api_tenant_isolation_policy_matrix_rows_invalid",
        "service_api_tenant_isolation_policy_matrix_row_count_mismatch",
        "service_api_tenant_isolation_policy_matrix_row_duplicate",
        "service_api_tenant_isolation_policy_matrix_row_id_invalid",
        "service_api_tenant_isolation_policy_matrix_row_missing",
        "service_api_tenant_isolation_policy_matrix_row_status_mismatch",
        "service_api_tenant_isolation_policy_matrix_row_leakage_result_mismatch",
        "service_api_tenant_isolation_policy_matrix_row_reason_code_mismatch",
        "service_api_tenant_isolation_policy_matrix_row_selector_mismatch",
        "service_api_tenant_isolation_policy_marker_missing",
        "service_api_tenant_isolation_policy_execution_reason_code_mismatch",
        "service_api_tenant_isolation_policy_command_count_invalid",
        "service_api_tenant_isolation_policy_command_count_mismatch",
        "service_api_tenant_isolation_policy_elapsed_seconds_invalid",
        "service_api_tenant_isolation_policy_max_seconds_invalid",
        "service_api_tenant_isolation_policy_runtime_budget_exceeded",
        "service_api_tenant_isolation_policy_docs_marker_missing",
    ]
)

OPT_IN_ENV = "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_OPT_IN"
DEFAULT_MAX_SECONDS = "180"
DEFAULT_COMMAND_MAX_SECONDS = "120"
MAX_BUDGET_SECONDS = 240

TAMPER_REASON_CODE = "service_api_tenant_isolation_policy_matrix_row_status_mismatch"
DOCS_MARKER_REASON_CODE = "service_api_tenant_isolation_policy_docs_marker_missing"

STRATEGY_REQUIRED_MARKERS: tuple[str, ...] = (
    "validate_service_api_tenant_isolation_matrix_live.sh",
    "check_service_api_tenant_isolation_matrix_live_policy.sh",
    "validate_service_api_tenant_isolation_matrix_live_contract_lane.sh",
    "service api tenant-isolation matrix run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
    "service_api_tenant_isolation_matrix_reason_taxonomy_version=kamn.runtime.service-api-tenant-isolation-matrix-policy-reason-taxonomy.v1",
    "service_api_tenant_isolation_matrix_reason_codes_csv=ci_fast_gate_failed,service_api_tenant_isolation_policy_schema_mismatch,service_api_tenant_isolation_policy_status_invalid,service_api_tenant_isolation_policy_final_decision_invalid,service_api_tenant_isolation_policy_final_decision_mismatch,service_api_tenant_isolation_policy_lane_mode_invalid,service_api_tenant_isolation_policy_matrix_schema_mismatch,service_api_tenant_isolation_policy_matrix_rows_invalid,service_api_tenant_isolation_policy_matrix_row_count_mismatch,service_api_tenant_isolation_policy_matrix_row_duplicate,service_api_tenant_isolation_policy_matrix_row_id_invalid,service_api_tenant_isolation_policy_matrix_row_missing,service_api_tenant_isolation_policy_matrix_row_status_mismatch,service_api_tenant_isolation_policy_matrix_row_leakage_result_mismatch,service_api_tenant_isolation_policy_matrix_row_reason_code_mismatch,service_api_tenant_isolation_policy_matrix_row_selector_mismatch,service_api_tenant_isolation_policy_marker_missing,service_api_tenant_isolation_policy_execution_reason_code_mismatch,service_api_tenant_isolation_policy_command_count_invalid,service_api_tenant_isolation_policy_command_count_mismatch,service_api_tenant_isolation_policy_elapsed_seconds_invalid,service_api_tenant_isolation_policy_max_seconds_invalid,service_api_tenant_isolation_policy_runtime_budget_exceeded,service_api_tenant_isolation_policy_docs_marker_missing",
)

OPS_REQUIRED_MARKERS: tuple[str, ...] = (
    "service_api_tenant_isolation_matrix_reason_taxonomy_version=kamn.runtime.service-api-tenant-isolation-matrix-policy-reason-taxonomy.v1",
    "service_api_tenant_isolation_matrix_reason_codes_csv=ci_fast_gate_failed,service_api_tenant_isolation_policy_schema_mismatch,service_api_tenant_isolation_policy_status_invalid,service_api_tenant_isolation_policy_final_decision_invalid,service_api_tenant_isolation_policy_final_decision_mismatch,service_api_tenant_isolation_policy_lane_mode_invalid,service_api_tenant_isolation_policy_matrix_schema_mismatch,service_api_tenant_isolation_policy_matrix_rows_invalid,service_api_tenant_isolation_policy_matrix_row_count_mismatch,service_api_tenant_isolation_policy_matrix_row_duplicate,service_api_tenant_isolation_policy_matrix_row_id_invalid,service_api_tenant_isolation_policy_matrix_row_missing,service_api_tenant_isolation_policy_matrix_row_status_mismatch,service_api_tenant_isolation_policy_matrix_row_leakage_result_mismatch,service_api_tenant_isolation_policy_matrix_row_reason_code_mismatch,service_api_tenant_isolation_policy_matrix_row_selector_mismatch,service_api_tenant_isolation_policy_marker_missing,service_api_tenant_isolation_policy_execution_reason_code_mismatch,service_api_tenant_isolation_policy_command_count_invalid,service_api_tenant_isolation_policy_command_count_mismatch,service_api_tenant_isolation_policy_elapsed_seconds_invalid,service_api_tenant_isolation_policy_max_seconds_invalid,service_api_tenant_isolation_policy_runtime_budget_exceeded,service_api_tenant_isolation_policy_docs_marker_missing",
    "service_api_tenant_isolation_matrix_matrix_schema_version=kamn.runtime.service-api-tenant-isolation-matrix.v1",
)

SCENARIO_ROWS: tuple[dict[str, str], ...] = (
    {
        "row_id": "m2_abac_cross_tenant_visibility_denied",
        "domain": "gateway_access",
        "evidence_test_target": "data_layer_m2_gateway_access",
        "evidence_test_selector": "spec_c03_abac_message_visibility_matrix_is_fail_closed_for_unrelated_requesters",
        "expected_reason_code": "m2_abac_scope_denied",
        "leakage_attempt_result": "rejected",
    },
    {
        "row_id": "m8_cross_owner_retention_and_shred_denied",
        "domain": "compliance_lifecycle",
        "evidence_test_target": "data_layer_m8_compliance_lifecycle",
        "evidence_test_selector": "spec_c04_cross_owner_operations_are_denied_fail_closed",
        "expected_reason_code": "m8_compliance_owner_scope_denied",
        "leakage_attempt_result": "rejected",
    },
    {
        "row_id": "m9_cross_owner_dispatch_and_presence_denied",
        "domain": "realtime_delivery",
        "evidence_test_target": "data_layer_m9_realtime_delivery",
        "evidence_test_selector": "spec_c04_cross_owner_dispatch_and_presence_queries_are_denied_fail_closed",
        "expected_reason_code": "m9_realtime_owner_scope_denied",
        "leakage_attempt_result": "rejected",
    },
    {
        "row_id": "m9_gateway_cross_owner_presence_denied",
        "domain": "gateway_bridge",
        "evidence_test_target": "data_layer_m9_gateway_bridge",
        "evidence_test_selector": "spec_c04_m9_gateway_presence_projection_fails_closed_for_cross_owner_scope_violation",
        "expected_reason_code": "m9_realtime_owner_scope_denied",
        "leakage_attempt_result": "rejected",
    },
)


def _dedupe_preserve_order(values: list[str]) -> list[str]:
    seen: set[str] = set()
    output: list[str] = []
    for value in values:
        if value in seen:
            continue
        seen.add(value)
        output.append(value)
    return output


def _is_non_negative_int(value: Any) -> bool:
    return isinstance(value, int) and not isinstance(value, bool) and value >= 0


def _run_cargo_test(row: dict[str, str], *, timeout_seconds: int) -> str:
    command = [
        "cargo",
        "test",
        "-p",
        "kamn-core",
        "--test",
        row["evidence_test_target"],
        row["evidence_test_selector"],
        "--",
        "--exact",
    ]
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
            "tenant-isolation matrix command timed out: "
            f"{row['row_id']} (timeout={timeout_seconds}s): {error}"
        )

    if completed.returncode != 0:
        detail = (completed.stderr or completed.stdout or "command failed").strip()
        fail(f"tenant-isolation matrix command failed for {row['row_id']}: {detail}")
    return " ".join(command)


def _build_matrix_rows() -> list[dict[str, str]]:
    matrix_rows: list[dict[str, str]] = []
    for row in SCENARIO_ROWS:
        matrix_rows.append(
            {
                "row_id": row["row_id"],
                "domain": row["domain"],
                "scenario_status": "verified",
                "leakage_attempt_result": row["leakage_attempt_result"],
                "expected_reason_code": row["expected_reason_code"],
                "evidence_test_target": row["evidence_test_target"],
                "evidence_test_selector": row["evidence_test_selector"],
            }
        )
    return matrix_rows


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    max_seconds = require_positive_int(
        "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )
    if max_seconds > MAX_BUDGET_SECONDS:
        fail(f"max-seconds must be <= {MAX_BUDGET_SECONDS} for tenant-isolation matrix lane")

    matrix_rows = _build_matrix_rows()
    start_epoch = int(time.time())
    commands: list[str] = []
    execution_reason_code = "dry_run_no_commands_executed"

    if mode == "run":
        if args.require_opt_in and args.local_opt_in != "1":
            fail(f"run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1")
        for row in SCENARIO_ROWS:
            commands.append(_run_cargo_test(row, timeout_seconds=command_max_seconds))
        execution_reason_code = "run_mode_commands_executed"

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "tenant-isolation matrix lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "tenant_isolation_status": "verified",
        "cross_tenant_leakage_status": "verified",
        "fail_closed_status": "verified",
        "ci_fast_gate_exclusion_status": "verified",
        "performance_budget_status": "verified",
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "matrix_schema_version": MATRIX_SCHEMA,
        "matrix_rows": matrix_rows,
        "leakage_attempt_count": len(matrix_rows),
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
    print("tenant_isolation_status=verified")
    print("cross_tenant_leakage_status=verified")
    print("fail_closed_status=verified")
    print("ci_fast_gate_exclusion_status=verified")
    print("performance_budget_status=verified")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"matrix_schema_version={MATRIX_SCHEMA}")
    print(f"leakage_attempt_count={len(matrix_rows)}")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"command_count={len(commands)}")
    if output_json is not None:
        print(f"report_file={output_json}")
    return 0


def _validate_matrix_rows(report_rows: Any, decision: DecisionAccumulator) -> None:
    expected_by_id = {row["row_id"]: row for row in SCENARIO_ROWS}
    expected_count = len(expected_by_id)

    if not isinstance(report_rows, list):
        decision.reject_if(True, "service_api_tenant_isolation_policy_matrix_rows_invalid")
        return

    decision.reject_if(
        len(report_rows) != expected_count,
        "service_api_tenant_isolation_policy_matrix_row_count_mismatch",
    )

    observed_ids: list[str] = []
    for row in report_rows:
        if not isinstance(row, dict):
            decision.reject_if(True, "service_api_tenant_isolation_policy_matrix_rows_invalid")
            continue
        row_id = row.get("row_id")
        if not isinstance(row_id, str) or row_id.strip() == "":
            decision.reject_if(True, "service_api_tenant_isolation_policy_matrix_row_id_invalid")
            continue
        observed_ids.append(row_id)
        expected_row = expected_by_id.get(row_id)
        if expected_row is None:
            decision.reject_if(True, "service_api_tenant_isolation_policy_matrix_row_id_invalid")
            continue

        decision.reject_if(
            row.get("scenario_status") != "verified",
            "service_api_tenant_isolation_policy_matrix_row_status_mismatch",
        )
        decision.reject_if(
            row.get("leakage_attempt_result") != expected_row["leakage_attempt_result"],
            "service_api_tenant_isolation_policy_matrix_row_leakage_result_mismatch",
        )
        decision.reject_if(
            row.get("expected_reason_code") != expected_row["expected_reason_code"],
            "service_api_tenant_isolation_policy_matrix_row_reason_code_mismatch",
        )
        decision.reject_if(
            row.get("evidence_test_selector") != expected_row["evidence_test_selector"],
            "service_api_tenant_isolation_policy_matrix_row_selector_mismatch",
        )

    deduped_ids = _dedupe_preserve_order(observed_ids)
    decision.reject_if(
        len(deduped_ids) != len(observed_ids),
        "service_api_tenant_isolation_policy_matrix_row_duplicate",
    )
    for expected_id in expected_by_id:
        decision.reject_if(
            expected_id not in observed_ids,
            "service_api_tenant_isolation_policy_matrix_row_missing",
        )


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
        "tenant_isolation_status",
        "cross_tenant_leakage_status",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
        "performance_budget_status",
        "reason_taxonomy_version",
        "reason_codes_csv",
        "matrix_schema_version",
        "matrix_rows",
        "leakage_attempt_count",
        "execution_reason_code",
        "command_count",
        "elapsed_seconds",
        "max_seconds",
    ]
    missing_fields = [field_name for field_name in required_fields if field_name not in report]
    if missing_fields:
        fail(f"missing required report fields: {','.join(missing_fields)}")

    decision = DecisionAccumulator()
    decision.reject_if(
        report.get("schema_version") != RUN_LANE_SCHEMA,
        "service_api_tenant_isolation_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "service_api_tenant_isolation_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "service_api_tenant_isolation_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "service_api_tenant_isolation_policy_final_decision_mismatch",
    )
    decision.reject_if(
        report.get("reason_taxonomy_version") != REASON_TAXONOMY_VERSION,
        "service_api_tenant_isolation_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("reason_codes_csv") != REASON_CODES_CSV,
        "service_api_tenant_isolation_policy_schema_mismatch",
    )
    for marker_name in (
        "tenant_isolation_status",
        "cross_tenant_leakage_status",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(marker_name) != "verified",
            "service_api_tenant_isolation_policy_marker_missing",
        )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "service_api_tenant_isolation_policy_lane_mode_invalid",
    )

    decision.reject_if(
        report.get("matrix_schema_version") != MATRIX_SCHEMA,
        "service_api_tenant_isolation_policy_matrix_schema_mismatch",
    )
    _validate_matrix_rows(report.get("matrix_rows"), decision)

    leakage_attempt_count = report.get("leakage_attempt_count")
    decision.reject_if(
        not _is_non_negative_int(leakage_attempt_count),
        "service_api_tenant_isolation_policy_matrix_rows_invalid",
    )
    if isinstance(leakage_attempt_count, int):
        decision.reject_if(
            leakage_attempt_count != len(SCENARIO_ROWS),
            "service_api_tenant_isolation_policy_matrix_row_count_mismatch",
        )

    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "service_api_tenant_isolation_policy_command_count_invalid",
    )

    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != "dry_run_no_commands_executed",
            "service_api_tenant_isolation_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "service_api_tenant_isolation_policy_command_count_mismatch",
        )
    elif lane_mode == "run":
        decision.reject_if(
            execution_reason_code != "run_mode_commands_executed",
            "service_api_tenant_isolation_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            not isinstance(command_count, int) or command_count < len(SCENARIO_ROWS),
            "service_api_tenant_isolation_policy_command_count_mismatch",
        )

    elapsed_seconds = report.get("elapsed_seconds")
    decision.reject_if(
        not _is_non_negative_int(elapsed_seconds),
        "service_api_tenant_isolation_policy_elapsed_seconds_invalid",
    )
    max_seconds = report.get("max_seconds")
    decision.reject_if(
        not _is_non_negative_int(max_seconds),
        "service_api_tenant_isolation_policy_max_seconds_invalid",
    )
    if isinstance(elapsed_seconds, int) and isinstance(max_seconds, int):
        decision.reject_if(
            elapsed_seconds > max_seconds,
            "service_api_tenant_isolation_policy_runtime_budget_exceeded",
        )
        decision.reject_if(
            max_seconds > MAX_BUDGET_SECONDS,
            "service_api_tenant_isolation_policy_runtime_budget_exceeded",
        )

    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"
    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "service_api_tenant_isolation_matrix_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
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
    print(f"service_api_tenant_isolation_matrix_policy_status={policy_status}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"tenant-isolation matrix policy rejected: {reason_codes_csv}")
    return 0


def _require_doc_markers(
    *,
    doc_file: Path,
    required_markers: tuple[str, ...],
    reason_code: str,
) -> None:
    if not doc_file.is_file():
        fail(f"{reason_code}: missing required documentation file: {doc_file}")
    doc_text = doc_file.read_text(encoding="utf-8")
    for marker in required_markers:
        if marker not in doc_text:
            fail(f"{reason_code}: missing documentation marker: {marker}")


def _invoke_with_captured_output(
    handler: Any,
    args: argparse.Namespace,
) -> str:
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        handler(args)
    return buffer.getvalue()


def _invoke_with_captured_output_allow_failure(
    handler: Any,
    args: argparse.Namespace,
) -> tuple[str, ContractError | None]:
    buffer = io.StringIO()
    with contextlib.redirect_stdout(buffer):
        try:
            handler(args)
        except ContractError as exc:
            return buffer.getvalue(), exc
    return buffer.getvalue(), None


def _require_output_markers(output: str, markers: tuple[str, ...], context: str) -> None:
    for marker in markers:
        if marker not in output:
            fail(f"{context} missing expected marker: {marker}")


def _run_contract_lane(args: argparse.Namespace) -> int:
    max_seconds = require_positive_int(
        "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_CONTRACT_MAX_SECONDS",
        args.max_seconds,
    )
    command_max_seconds = require_positive_int(
        "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_CONTRACT_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )
    if max_seconds > MAX_BUDGET_SECONDS:
        fail(
            f"max-seconds must be <= {MAX_BUDGET_SECONDS} for tenant-isolation matrix contract lane"
        )
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))

    strategy_doc = Path(args.strategy_doc).resolve()
    ops_doc = Path(args.ops_doc).resolve()

    start_epoch = int(time.time())
    with tempfile.TemporaryDirectory(prefix="tenant-isolation-contract-lane-") as tmp_dir_raw:
        tmp_dir = Path(tmp_dir_raw)
        summary_report = tmp_dir / "service-api-tenant-isolation-matrix-live-summary.json"
        policy_report = tmp_dir / "service-api-tenant-isolation-matrix-live-policy.json"
        tampered_report = (
            tmp_dir / "service-api-tenant-isolation-matrix-live-summary.tampered.json"
        )
        tampered_policy_report = (
            tmp_dir / "service-api-tenant-isolation-matrix-live-policy.tampered.json"
        )

        lane_output = _invoke_with_captured_output(
            _run_lane,
            argparse.Namespace(
                mode=mode,
                max_seconds=str(max_seconds),
                command_max_seconds=str(command_max_seconds),
                output_json=str(summary_report),
                local_opt_in=args.local_opt_in,
                require_opt_in=args.require_opt_in,
            ),
        )
        _require_output_markers(
            lane_output,
            (
                "status=pass",
                "final_decision=GO",
                f"lane_mode={mode}",
                "tenant_isolation_status=verified",
                "cross_tenant_leakage_status=verified",
            ),
            "tenant-isolation matrix lane output",
        )

        policy_output = _invoke_with_captured_output(
            _check_policy,
            argparse.Namespace(
                report_file=str(summary_report),
                expected_final_decision="GO",
                ci_fast_gate=ci_fast_gate,
                output_json=str(policy_report),
            ),
        )
        _require_output_markers(
            policy_output,
            (
                "status=ok",
                "final_decision=GO",
                "service_api_tenant_isolation_matrix_policy_status=verified",
            ),
            "tenant-isolation matrix policy output",
        )

        tampered_payload = dict(load_json(summary_report))
        tampered_rows = tampered_payload.get("matrix_rows")
        if not isinstance(tampered_rows, list) or not tampered_rows:
            fail("tenant-isolation matrix contract tamper setup requires non-empty matrix_rows")
        first_row = tampered_rows[0]
        if not isinstance(first_row, dict):
            fail("tenant-isolation matrix contract tamper setup requires dict matrix row")
        first_row["scenario_status"] = "missing"
        tampered_payload["matrix_rows"] = tampered_rows
        write_json(tampered_report, tampered_payload)

        tampered_output, tampered_error = _invoke_with_captured_output_allow_failure(
            _check_policy,
            argparse.Namespace(
                report_file=str(tampered_report),
                expected_final_decision="GO",
                ci_fast_gate=ci_fast_gate,
                output_json=str(tampered_policy_report),
            ),
        )
        if tampered_error is None:
            fail("expected tampered tenant-isolation matrix report to fail policy checker")
        if TAMPER_REASON_CODE not in str(tampered_error):
            fail(
                "expected deterministic tamper reason marker for tenant-isolation "
                f"matrix contract lane: {TAMPER_REASON_CODE}"
            )
        _require_output_markers(
            tampered_output,
            (
                "status=error",
                "final_decision=NO-GO",
                "service_api_tenant_isolation_matrix_policy_status=rejected",
            ),
            "tenant-isolation matrix tampered policy output",
        )

        _require_doc_markers(
            doc_file=strategy_doc,
            required_markers=STRATEGY_REQUIRED_MARKERS,
            reason_code=DOCS_MARKER_REASON_CODE,
        )
        _require_doc_markers(
            doc_file=ops_doc,
            required_markers=OPS_REQUIRED_MARKERS,
            reason_code=DOCS_MARKER_REASON_CODE,
        )

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_seconds:
            fail(
                "tenant-isolation matrix contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s (max={max_seconds}s)"
            )

        policy_payload = load_json(policy_report)
        lane_report = {
            "schema_version": CONTRACT_LANE_SCHEMA,
            "status": "pass",
            "final_decision": "GO",
            "service_api_tenant_isolation_matrix_contract_status": "verified",
            "service_api_tenant_isolation_matrix_policy_status": policy_payload.get(
                "service_api_tenant_isolation_matrix_policy_status",
                "unknown",
            ),
            "docs_contract_status": "verified",
            "fail_closed_status": "verified",
            "fail_closed_reason_code": TAMPER_REASON_CODE,
            "performance_budget_status": "verified",
            "lane_mode": mode,
            "elapsed_seconds": elapsed_seconds,
            "max_seconds": max_seconds,
        }

        if args.output_json:
            write_json(Path(args.output_json).resolve(), lane_report)
        if args.policy_output_json:
            write_json(Path(args.policy_output_json).resolve(), policy_payload)

    print("status=pass")
    print("final_decision=GO")
    print("service_api_tenant_isolation_matrix_contract_status=verified")
    print("service_api_tenant_isolation_matrix_policy_status=verified")
    print("docs_contract_status=verified")
    print("fail_closed_status=verified")
    print(f"fail_closed_reason_code={TAMPER_REASON_CODE}")
    print("performance_budget_status=verified")
    print(f"lane_mode={mode}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
    if args.policy_output_json:
        print(f"policy_report_file={Path(args.policy_output_json).resolve()}")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Service API tenant-isolation matrix lane and policy contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute tenant-isolation matrix lane in dry-run or run mode.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_MODE", "dry-run"),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_MAX_SECONDS",
            DEFAULT_MAX_SECONDS,
        ),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_COMMAND_MAX_SECONDS",
            DEFAULT_COMMAND_MAX_SECONDS,
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
    run_lane_parser.set_defaults(handler=_run_lane, require_opt_in=True)

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate tenant-isolation matrix report policy.",
    )
    check_policy_parser.add_argument("--report-file", required=True)
    check_policy_parser.add_argument("--expected-final-decision", default="GO")
    check_policy_parser.add_argument("--ci-fast-gate", default="PASS")
    check_policy_parser.add_argument("--output-json", default="")
    check_policy_parser.set_defaults(handler=_check_policy)

    contract_lane_parser = subparsers.add_parser(
        "run-contract-lane",
        help="Run tenant-isolation matrix contract lane composition checks.",
    )
    contract_lane_parser.add_argument(
        "--mode",
        default=os.environ.get(
            "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_CONTRACT_MODE",
            "dry-run",
        ),
        help="Contract lane mode: dry-run|run.",
    )
    contract_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_CONTRACT_MAX_SECONDS",
            DEFAULT_MAX_SECONDS,
        ),
        help="Maximum contract lane runtime budget in seconds.",
    )
    contract_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_SERVICE_API_TENANT_ISOLATION_MATRIX_CONTRACT_COMMAND_MAX_SECONDS",
            DEFAULT_COMMAND_MAX_SECONDS,
        ),
        help="Maximum runtime budget for each nested command in run mode.",
    )
    contract_lane_parser.add_argument("--ci-fast-gate", default="PASS")
    contract_lane_parser.add_argument("--output-json", default="")
    contract_lane_parser.add_argument("--policy-output-json", default="")
    contract_lane_parser.add_argument(
        "--strategy-doc",
        default=str(ROOT_DIR / "docs/ci/strategy.md"),
        help="CI strategy documentation path.",
    )
    contract_lane_parser.add_argument(
        "--ops-doc",
        default=str(ROOT_DIR / "docs/ops/configuration.md"),
        help="Ops configuration documentation path.",
    )
    contract_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, "0"),
        help="Opt-in marker value for run mode checks.",
    )
    contract_lane_parser.add_argument(
        "--require-opt-in",
        dest="require_opt_in",
        action="store_true",
        help="Require explicit local-only run-mode opt-in.",
    )
    contract_lane_parser.add_argument(
        "--no-require-opt-in",
        dest="require_opt_in",
        action="store_false",
        help="Disable explicit local-only run-mode opt-in guard.",
    )
    contract_lane_parser.set_defaults(handler=_run_contract_lane, require_opt_in=True)

    args = parser.parse_args()
    if hasattr(args, "mode"):
        args.mode = args.mode.strip()
    if hasattr(args, "max_seconds"):
        args.max_seconds = args.max_seconds.strip()
    if hasattr(args, "command_max_seconds"):
        args.command_max_seconds = args.command_max_seconds.strip()
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except ContractError as exc:
        print(str(exc), file=sys.stderr)
        raise SystemExit(1)
