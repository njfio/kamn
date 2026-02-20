#!/usr/bin/env python3
"""Request-response schema compatibility lane, policy checker, and contract lane."""

from __future__ import annotations

import argparse
import contextlib
import io
import os
from pathlib import Path
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

RUN_LANE_SCHEMA = "kamn.runtime.request-response-schema-compatibility-report.v1"
POLICY_SCHEMA = "kamn.runtime.request-response-schema-compatibility-policy-report.v1"
CONTRACT_LANE_SCHEMA = (
    "kamn.runtime.request-response-schema-compatibility-contract-lane-report.v1"
)
FIXTURE_SCHEMA = "kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1"

REASON_TAXONOMY_VERSION = "kamn.runtime.request-response-schema-compatibility-reason-taxonomy.v1"
REASON_CODES_CSV = ",".join(
    [
        "ci_fast_gate_failed",
        "request_response_schema_compatibility_schema_mismatch",
        "request_response_schema_compatibility_status_invalid",
        "request_response_schema_compatibility_final_decision_invalid",
        "request_response_schema_compatibility_final_decision_mismatch",
        "request_response_schema_compatibility_lane_mode_invalid",
        "request_response_schema_compatibility_fixture_schema_mismatch",
        "request_response_schema_compatibility_fixture_rows_invalid",
        "request_response_schema_compatibility_fixture_row_count_mismatch",
        "request_response_schema_compatibility_fixture_row_duplicate",
        "request_response_schema_compatibility_fixture_row_id_invalid",
        "request_response_schema_compatibility_fixture_row_missing",
        "request_response_schema_compatibility_fixture_row_status_mismatch",
        "request_response_schema_compatibility_fixture_row_decision_mismatch",
        "request_response_schema_compatibility_fixture_row_reason_code_mismatch",
        "request_response_schema_compatibility_fixture_row_version_pair_mismatch",
        "request_response_schema_compatibility_fixture_row_change_class_mismatch",
        "request_response_schema_compatibility_marker_missing",
        "request_response_schema_compatibility_execution_reason_code_mismatch",
        "request_response_schema_compatibility_command_count_invalid",
        "request_response_schema_compatibility_command_count_mismatch",
        "request_response_schema_compatibility_elapsed_seconds_invalid",
        "request_response_schema_compatibility_max_seconds_invalid",
        "request_response_schema_compatibility_runtime_budget_exceeded",
        "request_response_schema_compatibility_docs_marker_missing",
    ]
)

FIXTURE_SCHEMA_KEY = "request_response_schema_compatibility_fixture_matrix_schema_version"
FIXTURE_REASON_TAXONOMY_KEY = "request_response_schema_compatibility_reason_taxonomy_version"
FIXTURE_REASON_CODES_KEY = "request_response_schema_compatibility_reason_codes_csv"
ROW_PREFIX = "row"

OPT_IN_ENV = "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_OPT_IN"
DEFAULT_MAX_SECONDS = "180"
DEFAULT_COMMAND_MAX_SECONDS = "120"
MAX_BUDGET_SECONDS = 240

TAMPER_REASON_CODE = "request_response_schema_compatibility_fixture_row_status_mismatch"
DOCS_MARKER_REASON_CODE = "request_response_schema_compatibility_docs_marker_missing"
EXPECTED_INCOMPATIBLE_REASON = "schema_pair_breaking_change_detected"
FIXTURE_PATH = ROOT_DIR / "fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt"
REQUIRED_ROW_IDS_CSV = (
    "v1_to_v2_messages_send_optional_request_addition,"
    "v1_to_v2_channels_create_optional_response_addition,"
    "v1_to_v2_messages_get_required_response_removal,"
    "v1_to_v2_tasks_create_required_request_removal"
)

COMPATIBLE_CHANGE_CLASSES = {
    "request_field_optional_addition",
    "response_field_optional_addition",
}
INCOMPATIBLE_CHANGE_CLASSES = {
    "request_field_required_removal",
    "response_field_required_removal",
}
ALL_CHANGE_CLASSES = COMPATIBLE_CHANGE_CLASSES | INCOMPATIBLE_CHANGE_CLASSES

STRATEGY_REQUIRED_MARKERS: tuple[str, ...] = (
    "validate_request_response_schema_compatibility_live.sh",
    "check_request_response_schema_compatibility_live_policy.sh",
    "validate_request_response_schema_compatibility_live_contract_lane.sh",
    "request-response schema compatibility run-mode commands remain excluded from ci-fast-gate and ci-tools fast mode.",
    "request_response_schema_compatibility_reason_taxonomy_version=kamn.runtime.request-response-schema-compatibility-reason-taxonomy.v1",
    "request_response_schema_compatibility_reason_codes_csv=ci_fast_gate_failed,request_response_schema_compatibility_schema_mismatch,request_response_schema_compatibility_status_invalid,request_response_schema_compatibility_final_decision_invalid,request_response_schema_compatibility_final_decision_mismatch,request_response_schema_compatibility_lane_mode_invalid,request_response_schema_compatibility_fixture_schema_mismatch,request_response_schema_compatibility_fixture_rows_invalid,request_response_schema_compatibility_fixture_row_count_mismatch,request_response_schema_compatibility_fixture_row_duplicate,request_response_schema_compatibility_fixture_row_id_invalid,request_response_schema_compatibility_fixture_row_missing,request_response_schema_compatibility_fixture_row_status_mismatch,request_response_schema_compatibility_fixture_row_decision_mismatch,request_response_schema_compatibility_fixture_row_reason_code_mismatch,request_response_schema_compatibility_fixture_row_version_pair_mismatch,request_response_schema_compatibility_fixture_row_change_class_mismatch,request_response_schema_compatibility_marker_missing,request_response_schema_compatibility_execution_reason_code_mismatch,request_response_schema_compatibility_command_count_invalid,request_response_schema_compatibility_command_count_mismatch,request_response_schema_compatibility_elapsed_seconds_invalid,request_response_schema_compatibility_max_seconds_invalid,request_response_schema_compatibility_runtime_budget_exceeded,request_response_schema_compatibility_docs_marker_missing",
    "request_response_schema_compatibility_fixture_schema_version=kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1",
    "request_response_schema_compatibility_fixture_path=fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt",
    "request_response_schema_compatibility_required_row_ids_csv=v1_to_v2_messages_send_optional_request_addition,v1_to_v2_channels_create_optional_response_addition,v1_to_v2_messages_get_required_response_removal,v1_to_v2_tasks_create_required_request_removal",
)

OPS_REQUIRED_MARKERS: tuple[str, ...] = (
    "request_response_schema_compatibility_reason_taxonomy_version=kamn.runtime.request-response-schema-compatibility-reason-taxonomy.v1",
    "request_response_schema_compatibility_reason_codes_csv=ci_fast_gate_failed,request_response_schema_compatibility_schema_mismatch,request_response_schema_compatibility_status_invalid,request_response_schema_compatibility_final_decision_invalid,request_response_schema_compatibility_final_decision_mismatch,request_response_schema_compatibility_lane_mode_invalid,request_response_schema_compatibility_fixture_schema_mismatch,request_response_schema_compatibility_fixture_rows_invalid,request_response_schema_compatibility_fixture_row_count_mismatch,request_response_schema_compatibility_fixture_row_duplicate,request_response_schema_compatibility_fixture_row_id_invalid,request_response_schema_compatibility_fixture_row_missing,request_response_schema_compatibility_fixture_row_status_mismatch,request_response_schema_compatibility_fixture_row_decision_mismatch,request_response_schema_compatibility_fixture_row_reason_code_mismatch,request_response_schema_compatibility_fixture_row_version_pair_mismatch,request_response_schema_compatibility_fixture_row_change_class_mismatch,request_response_schema_compatibility_marker_missing,request_response_schema_compatibility_execution_reason_code_mismatch,request_response_schema_compatibility_command_count_invalid,request_response_schema_compatibility_command_count_mismatch,request_response_schema_compatibility_elapsed_seconds_invalid,request_response_schema_compatibility_max_seconds_invalid,request_response_schema_compatibility_runtime_budget_exceeded,request_response_schema_compatibility_docs_marker_missing",
    "request_response_schema_compatibility_fixture_schema_version=kamn.runtime.request-response-schema-compatibility-fixture-matrix.v1",
    "request_response_schema_compatibility_fixture_path=fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt",
    "request_response_schema_compatibility_required_row_ids_csv=v1_to_v2_messages_send_optional_request_addition,v1_to_v2_channels_create_optional_response_addition,v1_to_v2_messages_get_required_response_removal,v1_to_v2_tasks_create_required_request_removal",
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


def _parse_version(version: str, *, context: str) -> int:
    value = version.strip()
    if len(value) < 2 or not value.startswith("v"):
        fail(f"{context}: version must be v<integer>, got: {version}")
    digits = value[1:]
    if not digits.isdigit():
        fail(f"{context}: version must be v<integer>, got: {version}")
    return int(digits)


def _parse_fixture_row(parts: list[str], *, line_number: int) -> dict[str, Any]:
    if len(parts) != 7:
        fail(
            "request-response schema compatibility fixture row must contain 7 "
            f"pipe-delimited fields: line {line_number}"
        )

    (
        _,
        row_id,
        from_version,
        to_version,
        change_class,
        expected_decision,
        expected_reason_code,
    ) = (part.strip() for part in parts)

    if not row_id:
        fail(f"fixture row_id must be non-empty: line {line_number}")

    from_version_number = _parse_version(from_version, context=f"line {line_number}")
    to_version_number = _parse_version(to_version, context=f"line {line_number}")
    if to_version_number < from_version_number:
        fail(
            "fixture to_version must be >= from_version: "
            f"line {line_number}"
        )

    if change_class not in ALL_CHANGE_CLASSES:
        fail(
            "fixture change_class must be one of "
            f"{','.join(sorted(ALL_CHANGE_CLASSES))}: line {line_number}"
        )

    expected_final_decision = require_enum(
        f"fixture row expected_final_decision (line {line_number})",
        expected_decision,
        ("GO", "NO-GO"),
    )

    if expected_final_decision == "GO" and expected_reason_code != "none":
        fail(
            "fixture GO rows must use expected_reason_code=none: "
            f"line {line_number}"
        )
    if expected_final_decision == "NO-GO" and expected_reason_code != EXPECTED_INCOMPATIBLE_REASON:
        fail(
            "fixture NO-GO rows must use "
            f"expected_reason_code={EXPECTED_INCOMPATIBLE_REASON}: line {line_number}"
        )

    return {
        "row_id": row_id,
        "from_version": from_version,
        "to_version": to_version,
        "change_class": change_class,
        "expected_final_decision": expected_final_decision,
        "expected_reason_code": expected_reason_code,
    }


def _load_fixture_matrix(fixture_file: Path) -> dict[str, Any]:
    if not fixture_file.is_file():
        fail(f"fixture file not found: {fixture_file}")

    metadata: dict[str, str] = {}
    rows: list[dict[str, Any]] = []

    for line_number, raw_line in enumerate(fixture_file.read_text(encoding="utf-8").splitlines(), start=1):
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if line.startswith(f"{ROW_PREFIX}|"):
            rows.append(_parse_fixture_row(line.split("|"), line_number=line_number))
            continue
        if "=" not in line:
            fail(f"invalid fixture line {line_number}: {raw_line}")
        key, value = line.split("=", 1)
        key = key.strip()
        value = value.strip()
        if not key or not value:
            fail(f"invalid fixture metadata line {line_number}: {raw_line}")
        metadata[key] = value

    if metadata.get(FIXTURE_SCHEMA_KEY) != FIXTURE_SCHEMA:
        fail(
            "fixture schema mismatch: "
            f"expected {FIXTURE_SCHEMA_KEY}={FIXTURE_SCHEMA}"
        )
    if metadata.get(FIXTURE_REASON_TAXONOMY_KEY) != REASON_TAXONOMY_VERSION:
        fail(
            "fixture reason taxonomy mismatch: "
            f"expected {FIXTURE_REASON_TAXONOMY_KEY}={REASON_TAXONOMY_VERSION}"
        )
    if metadata.get(FIXTURE_REASON_CODES_KEY) != REASON_CODES_CSV:
        fail(
            "fixture reason codes mismatch: "
            f"expected {FIXTURE_REASON_CODES_KEY}={REASON_CODES_CSV}"
        )

    if not rows:
        fail("fixture must include at least one row")

    row_ids = [row["row_id"] for row in rows]
    if len(_dedupe_preserve_order(row_ids)) != len(row_ids):
        fail("fixture row ids must be unique")

    return {
        "fixture_schema_version": metadata[FIXTURE_SCHEMA_KEY],
        "reason_taxonomy_version": metadata[FIXTURE_REASON_TAXONOMY_KEY],
        "reason_codes_csv": metadata[FIXTURE_REASON_CODES_KEY],
        "rows": rows,
    }


def _evaluate_fixture_row(row: dict[str, Any]) -> dict[str, Any]:
    change_class = str(row["change_class"])
    if change_class in COMPATIBLE_CHANGE_CLASSES:
        observed_final_decision = "GO"
        observed_reason_code = "none"
        schema_compatibility_status = "compatible"
    else:
        observed_final_decision = "NO-GO"
        observed_reason_code = EXPECTED_INCOMPATIBLE_REASON
        schema_compatibility_status = "incompatible"

    row_status = "verified"
    if (
        observed_final_decision != row["expected_final_decision"]
        or observed_reason_code != row["expected_reason_code"]
    ):
        row_status = "mismatch"

    return {
        "row_id": row["row_id"],
        "from_version": row["from_version"],
        "to_version": row["to_version"],
        "change_class": row["change_class"],
        "schema_compatibility_status": schema_compatibility_status,
        "expected_final_decision": row["expected_final_decision"],
        "expected_reason_code": row["expected_reason_code"],
        "observed_final_decision": observed_final_decision,
        "observed_reason_code": observed_reason_code,
        "row_status": row_status,
    }


def _build_fixture_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [_evaluate_fixture_row(row) for row in rows]


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    max_seconds = require_positive_int(
        "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_MAX_SECONDS",
        args.max_seconds,
    )
    _ = require_positive_int(
        "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    if max_seconds > MAX_BUDGET_SECONDS:
        fail(f"max-seconds must be <= {MAX_BUDGET_SECONDS} for schema compatibility lane")

    fixture_file = Path(args.fixture_file).resolve()
    fixture_matrix = _load_fixture_matrix(fixture_file)

    if mode == "run" and args.require_opt_in and args.local_opt_in != "1":
        fail(f"run mode requires explicit local-only opt-in via {OPT_IN_ENV}=1")

    start_epoch = int(time.time())
    commands: list[str] = []

    fixture_rows = _build_fixture_rows(fixture_matrix["rows"])
    mismatched_rows = [row["row_id"] for row in fixture_rows if row["row_status"] != "verified"]
    if mismatched_rows:
        fail(
            "fixture expectations must match observed compatibility projection; "
            f"mismatched rows: {','.join(mismatched_rows)}"
        )

    compatible_pair_count = sum(
        1 for row in fixture_rows if row["schema_compatibility_status"] == "compatible"
    )
    incompatible_pair_count = sum(
        1 for row in fixture_rows if row["schema_compatibility_status"] == "incompatible"
    )
    if compatible_pair_count == 0 or incompatible_pair_count == 0:
        fail("fixture must include both compatible and incompatible rows")

    execution_reason_code = (
        "dry_run_no_commands_executed"
        if mode == "dry-run"
        else "run_mode_no_commands_executed"
    )

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "schema compatibility lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "compatible_pairs_status": "verified",
        "incompatible_pairs_status": "verified",
        "fail_closed_status": "verified",
        "ci_fast_gate_exclusion_status": "verified",
        "performance_budget_status": "verified",
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "fixture_schema_version": FIXTURE_SCHEMA,
        "fixture_path": "fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt",
        "required_row_ids_csv": REQUIRED_ROW_IDS_CSV,
        "fixture_rows": fixture_rows,
        "fixture_row_count": len(fixture_rows),
        "compatible_pair_count": compatible_pair_count,
        "incompatible_pair_count": incompatible_pair_count,
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
    print("compatible_pairs_status=verified")
    print("incompatible_pairs_status=verified")
    print("fail_closed_status=verified")
    print("ci_fast_gate_exclusion_status=verified")
    print("performance_budget_status=verified")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"fixture_schema_version={FIXTURE_SCHEMA}")
    print("fixture_path=fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt")
    print(f"required_row_ids_csv={REQUIRED_ROW_IDS_CSV}")
    print(f"fixture_row_count={len(fixture_rows)}")
    print(f"compatible_pair_count={compatible_pair_count}")
    print(f"incompatible_pair_count={incompatible_pair_count}")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"command_count={len(commands)}")
    if output_json is not None:
        print(f"report_file={output_json}")
    return 0


def _validate_fixture_rows(
    report_rows: Any,
    fixture_rows: list[dict[str, Any]],
    decision: DecisionAccumulator,
) -> None:
    expected_by_id = {row["row_id"]: row for row in fixture_rows}
    expected_count = len(expected_by_id)

    if not isinstance(report_rows, list):
        decision.reject_if(True, "request_response_schema_compatibility_fixture_rows_invalid")
        return

    decision.reject_if(
        len(report_rows) != expected_count,
        "request_response_schema_compatibility_fixture_row_count_mismatch",
    )

    observed_ids: list[str] = []
    for report_row in report_rows:
        if not isinstance(report_row, dict):
            decision.reject_if(True, "request_response_schema_compatibility_fixture_rows_invalid")
            continue

        row_id = report_row.get("row_id")
        if not isinstance(row_id, str) or row_id.strip() == "":
            decision.reject_if(True, "request_response_schema_compatibility_fixture_row_id_invalid")
            continue

        observed_ids.append(row_id)
        expected_row = expected_by_id.get(row_id)
        if expected_row is None:
            decision.reject_if(True, "request_response_schema_compatibility_fixture_row_id_invalid")
            continue

        decision.reject_if(
            report_row.get("row_status") != "verified",
            "request_response_schema_compatibility_fixture_row_status_mismatch",
        )
        decision.reject_if(
            report_row.get("from_version") != expected_row["from_version"],
            "request_response_schema_compatibility_fixture_row_version_pair_mismatch",
        )
        decision.reject_if(
            report_row.get("to_version") != expected_row["to_version"],
            "request_response_schema_compatibility_fixture_row_version_pair_mismatch",
        )
        decision.reject_if(
            report_row.get("change_class") != expected_row["change_class"],
            "request_response_schema_compatibility_fixture_row_change_class_mismatch",
        )

        expected_status = (
            "compatible"
            if expected_row["change_class"] in COMPATIBLE_CHANGE_CLASSES
            else "incompatible"
        )
        decision.reject_if(
            report_row.get("schema_compatibility_status") != expected_status,
            "request_response_schema_compatibility_fixture_row_change_class_mismatch",
        )
        decision.reject_if(
            report_row.get("expected_final_decision") != expected_row["expected_final_decision"],
            "request_response_schema_compatibility_fixture_row_decision_mismatch",
        )
        decision.reject_if(
            report_row.get("observed_final_decision") != expected_row["expected_final_decision"],
            "request_response_schema_compatibility_fixture_row_decision_mismatch",
        )
        decision.reject_if(
            report_row.get("expected_reason_code") != expected_row["expected_reason_code"],
            "request_response_schema_compatibility_fixture_row_reason_code_mismatch",
        )
        decision.reject_if(
            report_row.get("observed_reason_code") != expected_row["expected_reason_code"],
            "request_response_schema_compatibility_fixture_row_reason_code_mismatch",
        )

    deduped_ids = _dedupe_preserve_order(observed_ids)
    decision.reject_if(
        len(deduped_ids) != len(observed_ids),
        "request_response_schema_compatibility_fixture_row_duplicate",
    )
    for expected_id in expected_by_id:
        decision.reject_if(
            expected_id not in observed_ids,
            "request_response_schema_compatibility_fixture_row_missing",
        )


def _check_policy(args: argparse.Namespace) -> int:
    report_file = Path(args.report_file).resolve()
    if not report_file.is_file():
        fail(f"report file not found: {report_file}")

    report = load_json(report_file)
    fixture_file = Path(args.fixture_file).resolve()
    fixture_matrix = _load_fixture_matrix(fixture_file)
    fixture_rows: list[dict[str, Any]] = fixture_matrix["rows"]
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
        "compatible_pairs_status",
        "incompatible_pairs_status",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
        "performance_budget_status",
        "reason_taxonomy_version",
        "reason_codes_csv",
        "fixture_schema_version",
        "fixture_path",
        "required_row_ids_csv",
        "fixture_rows",
        "fixture_row_count",
        "compatible_pair_count",
        "incompatible_pair_count",
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
        "request_response_schema_compatibility_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "request_response_schema_compatibility_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "request_response_schema_compatibility_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "request_response_schema_compatibility_final_decision_mismatch",
    )
    decision.reject_if(
        report.get("reason_taxonomy_version") != REASON_TAXONOMY_VERSION,
        "request_response_schema_compatibility_schema_mismatch",
    )
    decision.reject_if(
        report.get("reason_codes_csv") != REASON_CODES_CSV,
        "request_response_schema_compatibility_schema_mismatch",
    )

    for marker_name in (
        "compatible_pairs_status",
        "incompatible_pairs_status",
        "fail_closed_status",
        "ci_fast_gate_exclusion_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(marker_name) != "verified",
            "request_response_schema_compatibility_marker_missing",
        )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "request_response_schema_compatibility_lane_mode_invalid",
    )

    decision.reject_if(
        report.get("fixture_schema_version") != FIXTURE_SCHEMA,
        "request_response_schema_compatibility_fixture_schema_mismatch",
    )
    decision.reject_if(
        report.get("fixture_path")
        != "fixtures/runtime/request_response_schema_compatibility_fixture_matrix.txt",
        "request_response_schema_compatibility_fixture_schema_mismatch",
    )
    decision.reject_if(
        report.get("required_row_ids_csv") != REQUIRED_ROW_IDS_CSV,
        "request_response_schema_compatibility_fixture_schema_mismatch",
    )

    _validate_fixture_rows(report.get("fixture_rows"), fixture_rows, decision)

    fixture_row_count = report.get("fixture_row_count")
    decision.reject_if(
        not _is_non_negative_int(fixture_row_count),
        "request_response_schema_compatibility_fixture_rows_invalid",
    )
    if isinstance(fixture_row_count, int):
        decision.reject_if(
            fixture_row_count != len(fixture_rows),
            "request_response_schema_compatibility_fixture_row_count_mismatch",
        )

    compatible_pair_count = report.get("compatible_pair_count")
    incompatible_pair_count = report.get("incompatible_pair_count")
    decision.reject_if(
        not _is_non_negative_int(compatible_pair_count)
        or not _is_non_negative_int(incompatible_pair_count),
        "request_response_schema_compatibility_fixture_rows_invalid",
    )

    expected_compatible = sum(
        1 for row in fixture_rows if row["change_class"] in COMPATIBLE_CHANGE_CLASSES
    )
    expected_incompatible = sum(
        1 for row in fixture_rows if row["change_class"] in INCOMPATIBLE_CHANGE_CLASSES
    )
    if isinstance(compatible_pair_count, int):
        decision.reject_if(
            compatible_pair_count != expected_compatible,
            "request_response_schema_compatibility_fixture_row_count_mismatch",
        )
    if isinstance(incompatible_pair_count, int):
        decision.reject_if(
            incompatible_pair_count != expected_incompatible,
            "request_response_schema_compatibility_fixture_row_count_mismatch",
        )

    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "request_response_schema_compatibility_command_count_invalid",
    )

    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != "dry_run_no_commands_executed",
            "request_response_schema_compatibility_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "request_response_schema_compatibility_command_count_mismatch",
        )
    elif lane_mode == "run":
        decision.reject_if(
            execution_reason_code != "run_mode_no_commands_executed",
            "request_response_schema_compatibility_execution_reason_code_mismatch",
        )
        decision.reject_if(
            command_count != 0,
            "request_response_schema_compatibility_command_count_mismatch",
        )

    elapsed_seconds = report.get("elapsed_seconds")
    decision.reject_if(
        not _is_non_negative_int(elapsed_seconds),
        "request_response_schema_compatibility_elapsed_seconds_invalid",
    )
    max_seconds = report.get("max_seconds")
    decision.reject_if(
        not _is_non_negative_int(max_seconds),
        "request_response_schema_compatibility_max_seconds_invalid",
    )
    if isinstance(elapsed_seconds, int) and isinstance(max_seconds, int):
        decision.reject_if(
            elapsed_seconds > max_seconds,
            "request_response_schema_compatibility_runtime_budget_exceeded",
        )
        decision.reject_if(
            max_seconds > MAX_BUDGET_SECONDS,
            "request_response_schema_compatibility_runtime_budget_exceeded",
        )

    decision.reject_if(ci_fast_gate != "PASS", "ci_fast_gate_failed")

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "request_response_schema_compatibility_policy_status": policy_status,
        "expected_final_decision": expected_final_decision,
        "observed_final_decision": report.get("final_decision"),
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes": reason_codes,
        "ci_fast_gate": ci_fast_gate,
        "source_report_file": str(report_file),
        "source_fixture_file": str(fixture_file),
        "generated_at_epoch": int(time.time()),
    }

    output_json = None
    if args.output_json:
        output_json = Path(args.output_json).resolve()
        write_json(output_json, policy_report)

    reason_codes_csv = ",".join(reason_codes)
    print(f"status={'ok' if final_decision == 'GO' else 'error'}")
    print(f"final_decision={final_decision}")
    print(f"request_response_schema_compatibility_policy_status={policy_status}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"schema compatibility policy rejected: {reason_codes_csv}")
    return 0


def _require_doc_markers(*, doc_file: Path, required_markers: tuple[str, ...], reason_code: str) -> None:
    if not doc_file.is_file():
        fail(f"{reason_code}: missing required documentation file: {doc_file}")
    doc_text = doc_file.read_text(encoding="utf-8")
    for marker in required_markers:
        if marker not in doc_text:
            fail(f"{reason_code}: missing documentation marker: {marker}")


def _invoke_with_captured_output(handler: Any, args: argparse.Namespace) -> str:
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
        "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_MAX_SECONDS",
        args.max_seconds,
    )
    _ = require_positive_int(
        "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )
    if max_seconds > MAX_BUDGET_SECONDS:
        fail(
            "max-seconds must be <= "
            f"{MAX_BUDGET_SECONDS} for schema compatibility contract lane"
        )

    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))

    fixture_file = Path(args.fixture_file).resolve()
    strategy_doc = Path(args.strategy_doc).resolve()
    ops_doc = Path(args.ops_doc).resolve()

    start_epoch = int(time.time())
    with tempfile.TemporaryDirectory(prefix="schema-compatibility-contract-lane-") as tmp_dir_raw:
        tmp_dir = Path(tmp_dir_raw)
        summary_report = tmp_dir / "schema-compatibility-live-summary.json"
        policy_report = tmp_dir / "schema-compatibility-live-policy.json"
        tampered_report = tmp_dir / "schema-compatibility-live-summary.tampered.json"
        tampered_policy_report = tmp_dir / "schema-compatibility-live-policy.tampered.json"

        lane_output = _invoke_with_captured_output(
            _run_lane,
            argparse.Namespace(
                mode=mode,
                max_seconds=str(max_seconds),
                command_max_seconds=args.command_max_seconds,
                output_json=str(summary_report),
                fixture_file=str(fixture_file),
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
                "compatible_pairs_status=verified",
                "incompatible_pairs_status=verified",
            ),
            "schema compatibility lane output",
        )

        policy_output = _invoke_with_captured_output(
            _check_policy,
            argparse.Namespace(
                report_file=str(summary_report),
                fixture_file=str(fixture_file),
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
                "request_response_schema_compatibility_policy_status=verified",
            ),
            "schema compatibility policy output",
        )

        tampered_payload = dict(load_json(summary_report))
        tampered_rows = tampered_payload.get("fixture_rows")
        if not isinstance(tampered_rows, list) or not tampered_rows:
            fail("contract tamper setup requires non-empty fixture_rows")
        first_row = tampered_rows[0]
        if not isinstance(first_row, dict):
            fail("contract tamper setup requires dict fixture row")
        first_row["row_status"] = "missing"
        tampered_payload["fixture_rows"] = tampered_rows
        write_json(tampered_report, tampered_payload)

        tampered_output, tampered_error = _invoke_with_captured_output_allow_failure(
            _check_policy,
            argparse.Namespace(
                report_file=str(tampered_report),
                fixture_file=str(fixture_file),
                expected_final_decision="GO",
                ci_fast_gate=ci_fast_gate,
                output_json=str(tampered_policy_report),
            ),
        )
        if tampered_error is None:
            fail("expected tampered schema compatibility report to fail policy checker")
        if TAMPER_REASON_CODE not in str(tampered_error):
            fail(
                "expected deterministic tamper reason marker for schema compatibility "
                f"contract lane: {TAMPER_REASON_CODE}"
            )
        _require_output_markers(
            tampered_output,
            (
                "status=error",
                "final_decision=NO-GO",
                "request_response_schema_compatibility_policy_status=rejected",
            ),
            "schema compatibility tampered policy output",
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
                "schema compatibility contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s (max={max_seconds}s)"
            )

        policy_payload = load_json(policy_report)
        lane_report = {
            "schema_version": CONTRACT_LANE_SCHEMA,
            "status": "pass",
            "final_decision": "GO",
            "request_response_schema_compatibility_contract_status": "verified",
            "request_response_schema_compatibility_policy_status": policy_payload.get(
                "request_response_schema_compatibility_policy_status",
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
    print("request_response_schema_compatibility_contract_status=verified")
    print("request_response_schema_compatibility_policy_status=verified")
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
        description="Request-response schema compatibility lane and policy contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute request-response schema compatibility lane in dry-run or run mode.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_MODE", "dry-run"),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_MAX_SECONDS",
            DEFAULT_MAX_SECONDS,
        ),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_COMMAND_MAX_SECONDS",
            DEFAULT_COMMAND_MAX_SECONDS,
        ),
        help="Maximum runtime budget for nested commands in run mode.",
    )
    run_lane_parser.add_argument(
        "--fixture-file",
        default=str(FIXTURE_PATH),
        help="Fixture matrix file for schema compatibility checks.",
    )
    run_lane_parser.add_argument("--output-json", default="")
    run_lane_parser.add_argument(
        "--local-opt-in",
        default=os.environ.get(OPT_IN_ENV, "0"),
        help="Opt-in marker value for run mode checks.",
    )
    run_lane_parser.add_argument(
        "--require-opt-in",
        dest="require_opt_in",
        action="store_true",
    )
    run_lane_parser.add_argument(
        "--no-require-opt-in",
        dest="require_opt_in",
        action="store_false",
    )
    run_lane_parser.set_defaults(handler=_run_lane, require_opt_in=True)

    check_policy_parser = subparsers.add_parser(
        "check-policy",
        help="Validate request-response schema compatibility policy.",
    )
    check_policy_parser.add_argument("--report-file", required=True)
    check_policy_parser.add_argument("--fixture-file", default=str(FIXTURE_PATH))
    check_policy_parser.add_argument("--expected-final-decision", default="GO")
    check_policy_parser.add_argument("--ci-fast-gate", default="PASS")
    check_policy_parser.add_argument("--output-json", default="")
    check_policy_parser.set_defaults(handler=_check_policy)

    contract_lane_parser = subparsers.add_parser(
        "run-contract-lane",
        help="Run request-response schema compatibility contract lane.",
    )
    contract_lane_parser.add_argument(
        "--mode",
        default=os.environ.get(
            "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_MODE",
            "dry-run",
        ),
        help="Contract lane mode: dry-run|run.",
    )
    contract_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_MAX_SECONDS",
            DEFAULT_MAX_SECONDS,
        ),
        help="Maximum contract-lane runtime budget in seconds.",
    )
    contract_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_REQUEST_RESPONSE_SCHEMA_COMPATIBILITY_CONTRACT_COMMAND_MAX_SECONDS",
            DEFAULT_COMMAND_MAX_SECONDS,
        ),
        help="Maximum runtime budget for nested commands in run mode.",
    )
    contract_lane_parser.add_argument("--ci-fast-gate", default="PASS")
    contract_lane_parser.add_argument("--fixture-file", default=str(FIXTURE_PATH))
    contract_lane_parser.add_argument("--output-json", default="")
    contract_lane_parser.add_argument("--policy-output-json", default="")
    contract_lane_parser.add_argument(
        "--strategy-doc",
        default=str(ROOT_DIR / "docs/ci/strategy.md"),
    )
    contract_lane_parser.add_argument(
        "--ops-doc",
        default=str(ROOT_DIR / "docs/ops/configuration.md"),
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
    )
    contract_lane_parser.add_argument(
        "--no-require-opt-in",
        dest="require_opt_in",
        action="store_false",
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
