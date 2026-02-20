#!/usr/bin/env python3
"""API compatibility matrix local-heavy lane, policy checker, and contract lane."""

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

RUN_LANE_SCHEMA = "kamn.runtime.api-compatibility-matrix-local-heavy-live-report.v1"
POLICY_SCHEMA = "kamn.runtime.api-compatibility-matrix-local-heavy-live-policy-report.v1"
CONTRACT_LANE_SCHEMA = (
    "kamn.runtime.api-compatibility-matrix-local-heavy-live-contract-lane-report.v1"
)
FIXTURE_SCHEMA = "kamn.runtime.api-compatibility-matrix-local-heavy-fixture-matrix.v1"
ARTIFACT_SCHEMA = "kamn.runtime.api-compatibility-matrix-local-heavy-artifact-schema.v1"

REASON_TAXONOMY_VERSION = (
    "kamn.runtime.api-compatibility-matrix-local-heavy-policy-reason-taxonomy.v1"
)
REASON_CODES_CSV = ",".join(
    [
        "ci_fast_gate_failed",
        "api_compatibility_matrix_local_heavy_policy_schema_mismatch",
        "api_compatibility_matrix_local_heavy_policy_status_invalid",
        "api_compatibility_matrix_local_heavy_policy_final_decision_invalid",
        "api_compatibility_matrix_local_heavy_policy_final_decision_mismatch",
        "api_compatibility_matrix_local_heavy_policy_lane_mode_invalid",
        "api_compatibility_matrix_local_heavy_policy_artifact_schema_mismatch",
        "api_compatibility_matrix_local_heavy_policy_fixture_schema_mismatch",
        "api_compatibility_matrix_local_heavy_policy_fixture_rows_invalid",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_count_mismatch",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_duplicate",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_id_invalid",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_missing",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_status_mismatch",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_decision_mismatch",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_reason_code_mismatch",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_version_pair_mismatch",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_route_selector_mismatch",
        "api_compatibility_matrix_local_heavy_policy_fixture_row_change_class_mismatch",
        "api_compatibility_matrix_local_heavy_policy_marker_missing",
        "api_compatibility_matrix_local_heavy_policy_execution_reason_code_mismatch",
        "api_compatibility_matrix_local_heavy_policy_command_count_invalid",
        "api_compatibility_matrix_local_heavy_policy_command_count_mismatch",
        "api_compatibility_matrix_local_heavy_policy_elapsed_seconds_invalid",
        "api_compatibility_matrix_local_heavy_policy_max_seconds_invalid",
        "api_compatibility_matrix_local_heavy_policy_runtime_budget_exceeded",
        "api_compatibility_matrix_local_heavy_policy_local_heavy_opt_in_required",
        "api_compatibility_matrix_local_heavy_policy_local_heavy_scope_mismatch",
        "api_compatibility_matrix_local_heavy_policy_docs_marker_missing",
    ]
)

FIXTURE_SCHEMA_KEY = "api_compatibility_matrix_local_heavy_fixture_matrix_schema_version"
FIXTURE_REASON_TAXONOMY_KEY = "api_compatibility_matrix_local_heavy_reason_taxonomy_version"
FIXTURE_REASON_CODES_KEY = "api_compatibility_matrix_local_heavy_reason_codes_csv"
ROW_PREFIX = "row"

OPT_IN_ENV = "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_OPT_IN"
DEFAULT_MAX_SECONDS = "210"
DEFAULT_COMMAND_MAX_SECONDS = "120"
MAX_BUDGET_SECONDS = 300

DRY_RUN_REASON_CODE = "dry_run_no_commands_executed"
RUN_MODE_REASON_CODE = "local_heavy_projection_executed"
TAMPER_REASON_CODE = "api_compatibility_matrix_local_heavy_policy_fixture_row_status_mismatch"
DOCS_MARKER_REASON_CODE = "api_compatibility_matrix_local_heavy_policy_docs_marker_missing"
LOCAL_HEAVY_SCOPE_REASON_CODE = "api_compatibility_matrix_local_heavy_policy_local_heavy_scope_mismatch"
LOCAL_HEAVY_OPT_IN_REASON_CODE = (
    "api_compatibility_matrix_local_heavy_policy_local_heavy_opt_in_required"
)

FIXTURE_PATH = ROOT_DIR / "fixtures/runtime/api_compatibility_matrix_local_heavy_fixture_matrix.txt"
OPS_DOC_PATH = ROOT_DIR / "docs/ops/configuration.md"
REQUIRED_ROW_IDS_CSV = (
    "v1_to_v2_messages_send_optional_request_addition,"
    "v1_to_v2_channels_create_optional_response_addition,"
    "v1_to_v2_tasks_create_required_request_removal,"
    "v1_to_v2_messages_get_required_response_removal,"
    "v1_to_v2_messages_send_enum_variant_removal"
)

COMPATIBLE_CHANGE_CLASSES = {
    "request_field_optional_addition",
    "response_field_optional_addition",
}
INCOMPATIBLE_REASON_BY_CLASS = {
    "request_field_required_removal": "incompatible_request_breaking_change",
    "response_field_required_removal": "incompatible_response_breaking_change",
    "enum_variant_removal": "incompatible_enum_breaking_change",
}
ALL_CHANGE_CLASSES = COMPATIBLE_CHANGE_CLASSES | set(INCOMPATIBLE_REASON_BY_CLASS)

OPS_REQUIRED_MARKERS: tuple[str, ...] = (
    "validate_api_compatibility_matrix_local_heavy_live.sh",
    "check_api_compatibility_matrix_local_heavy_live_policy.sh",
    "validate_api_compatibility_matrix_local_heavy_live_contract_lane.sh",
    "api_compatibility_matrix_local_heavy_reason_taxonomy_version=kamn.runtime.api-compatibility-matrix-local-heavy-policy-reason-taxonomy.v1",
    "api_compatibility_matrix_local_heavy_reason_codes_csv=ci_fast_gate_failed,api_compatibility_matrix_local_heavy_policy_schema_mismatch,api_compatibility_matrix_local_heavy_policy_status_invalid,api_compatibility_matrix_local_heavy_policy_final_decision_invalid,api_compatibility_matrix_local_heavy_policy_final_decision_mismatch,api_compatibility_matrix_local_heavy_policy_lane_mode_invalid,api_compatibility_matrix_local_heavy_policy_artifact_schema_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_schema_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_rows_invalid,api_compatibility_matrix_local_heavy_policy_fixture_row_count_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_duplicate,api_compatibility_matrix_local_heavy_policy_fixture_row_id_invalid,api_compatibility_matrix_local_heavy_policy_fixture_row_missing,api_compatibility_matrix_local_heavy_policy_fixture_row_status_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_decision_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_reason_code_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_version_pair_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_route_selector_mismatch,api_compatibility_matrix_local_heavy_policy_fixture_row_change_class_mismatch,api_compatibility_matrix_local_heavy_policy_marker_missing,api_compatibility_matrix_local_heavy_policy_execution_reason_code_mismatch,api_compatibility_matrix_local_heavy_policy_command_count_invalid,api_compatibility_matrix_local_heavy_policy_command_count_mismatch,api_compatibility_matrix_local_heavy_policy_elapsed_seconds_invalid,api_compatibility_matrix_local_heavy_policy_max_seconds_invalid,api_compatibility_matrix_local_heavy_policy_runtime_budget_exceeded,api_compatibility_matrix_local_heavy_policy_local_heavy_opt_in_required,api_compatibility_matrix_local_heavy_policy_local_heavy_scope_mismatch,api_compatibility_matrix_local_heavy_policy_docs_marker_missing",
    "api_compatibility_matrix_local_heavy_fixture_schema_version=kamn.runtime.api-compatibility-matrix-local-heavy-fixture-matrix.v1",
    "api_compatibility_matrix_local_heavy_fixture_path=fixtures/runtime/api_compatibility_matrix_local_heavy_fixture_matrix.txt",
    "api_compatibility_matrix_local_heavy_required_row_ids_csv=v1_to_v2_messages_send_optional_request_addition,v1_to_v2_channels_create_optional_response_addition,v1_to_v2_tasks_create_required_request_removal,v1_to_v2_messages_get_required_response_removal,v1_to_v2_messages_send_enum_variant_removal",
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
    if len(parts) != 8:
        fail(
            "api compatibility matrix fixture row must contain 8 pipe-delimited fields: "
            f"line {line_number}"
        )

    (
        _,
        row_id,
        from_version,
        to_version,
        route_selector,
        change_class,
        expected_decision,
        expected_reason_code,
    ) = (part.strip() for part in parts)

    if not row_id:
        fail(f"fixture row_id must be non-empty: line {line_number}")
    if not route_selector:
        fail(f"fixture route_selector must be non-empty: line {line_number}")

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

    expected_reason_for_class = "none"
    if change_class in INCOMPATIBLE_REASON_BY_CLASS:
        expected_reason_for_class = INCOMPATIBLE_REASON_BY_CLASS[change_class]

    if expected_final_decision == "GO" and expected_reason_code != "none":
        fail(
            "fixture GO rows must use expected_reason_code=none: "
            f"line {line_number}"
        )
    if expected_final_decision == "NO-GO" and expected_reason_code != expected_reason_for_class:
        fail(
            "fixture NO-GO rows must use expected_reason_code="
            f"{expected_reason_for_class}: line {line_number}"
        )

    return {
        "row_id": row_id,
        "from_version": from_version,
        "to_version": to_version,
        "route_selector": route_selector,
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
        compatibility_status = "compatible"
    else:
        observed_final_decision = "NO-GO"
        observed_reason_code = INCOMPATIBLE_REASON_BY_CLASS[change_class]
        compatibility_status = "incompatible"

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
        "route_selector": row["route_selector"],
        "change_class": row["change_class"],
        "compatibility_status": compatibility_status,
        "expected_final_decision": row["expected_final_decision"],
        "expected_reason_code": row["expected_reason_code"],
        "observed_final_decision": observed_final_decision,
        "observed_reason_code": observed_reason_code,
        "row_status": row_status,
    }


def _build_matrix_rows(rows: list[dict[str, Any]]) -> list[dict[str, Any]]:
    return [_evaluate_fixture_row(row) for row in rows]


def _run_lane(args: argparse.Namespace) -> int:
    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))
    max_seconds = require_positive_int(
        "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_MAX_SECONDS",
        args.max_seconds,
    )
    _ = require_positive_int(
        "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )

    if max_seconds > MAX_BUDGET_SECONDS:
        fail(f"max-seconds must be <= {MAX_BUDGET_SECONDS} for local-heavy compatibility lane")

    fixture_file = Path(args.fixture_file).resolve()
    fixture_matrix = _load_fixture_matrix(fixture_file)

    if mode == "run" and args.require_opt_in and args.local_opt_in != "1":
        fail(
            "run mode requires explicit local-only opt-in via "
            f"{OPT_IN_ENV}=1 ({LOCAL_HEAVY_OPT_IN_REASON_CODE})"
        )

    if mode == "run" and ci_fast_gate != "FAIL":
        fail(
            "run mode must be scoped out of ci-fast-gate "
            f"({LOCAL_HEAVY_SCOPE_REASON_CODE})"
        )

    start_epoch = int(time.time())
    commands: list[str] = []

    matrix_rows = _build_matrix_rows(fixture_matrix["rows"])
    mismatched_rows = [row["row_id"] for row in matrix_rows if row["row_status"] != "verified"]
    if mismatched_rows:
        fail(
            "fixture expectations must match observed compatibility projection; "
            f"mismatched rows: {','.join(mismatched_rows)}"
        )

    compatible_row_count = sum(
        1 for row in matrix_rows if row["compatibility_status"] == "compatible"
    )
    incompatible_row_count = sum(
        1 for row in matrix_rows if row["compatibility_status"] == "incompatible"
    )

    if compatible_row_count == 0 or incompatible_row_count == 0:
        fail("fixture must include both compatible and incompatible rows")

    execution_reason_code = (
        DRY_RUN_REASON_CODE
        if mode == "dry-run"
        else RUN_MODE_REASON_CODE
    )

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        fail(
            "local-heavy compatibility matrix lane exceeded runtime budget: "
            f"{elapsed_seconds}s (max={max_seconds}s)"
        )

    report_payload = {
        "schema_version": RUN_LANE_SCHEMA,
        "status": "pass",
        "final_decision": "GO",
        "lane_mode": mode,
        "ci_fast_gate": ci_fast_gate,
        "matrix_artifact_status": "verified",
        "compatibility_class_projection_status": "verified",
        "local_heavy_scope_status": "verified",
        "fail_closed_status": "verified",
        "performance_budget_status": "verified",
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "artifact_schema_version": ARTIFACT_SCHEMA,
        "fixture_schema_version": FIXTURE_SCHEMA,
        "fixture_path": "fixtures/runtime/api_compatibility_matrix_local_heavy_fixture_matrix.txt",
        "required_row_ids_csv": REQUIRED_ROW_IDS_CSV,
        "matrix_rows": matrix_rows,
        "matrix_row_count": len(matrix_rows),
        "compatible_row_count": compatible_row_count,
        "incompatible_row_count": incompatible_row_count,
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
    print("matrix_artifact_status=verified")
    print("compatibility_class_projection_status=verified")
    print("local_heavy_scope_status=verified")
    print("fail_closed_status=verified")
    print("performance_budget_status=verified")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"artifact_schema_version={ARTIFACT_SCHEMA}")
    print(f"fixture_schema_version={FIXTURE_SCHEMA}")
    print("fixture_path=fixtures/runtime/api_compatibility_matrix_local_heavy_fixture_matrix.txt")
    print(f"required_row_ids_csv={REQUIRED_ROW_IDS_CSV}")
    print(f"matrix_row_count={len(matrix_rows)}")
    print(f"compatible_row_count={compatible_row_count}")
    print(f"incompatible_row_count={incompatible_row_count}")
    print(f"execution_reason_code={execution_reason_code}")
    print(f"command_count={len(commands)}")
    if output_json is not None:
        print(f"report_file={output_json}")
    return 0


def _validate_matrix_rows(
    report_rows: Any,
    fixture_rows: list[dict[str, Any]],
    decision: DecisionAccumulator,
) -> None:
    expected_by_id = {row["row_id"]: row for row in fixture_rows}

    if not isinstance(report_rows, list):
        decision.reject_if(True, "api_compatibility_matrix_local_heavy_policy_fixture_rows_invalid")
        return

    decision.reject_if(
        len(report_rows) != len(expected_by_id),
        "api_compatibility_matrix_local_heavy_policy_fixture_row_count_mismatch",
    )

    observed_ids: list[str] = []
    for report_row in report_rows:
        if not isinstance(report_row, dict):
            decision.reject_if(True, "api_compatibility_matrix_local_heavy_policy_fixture_rows_invalid")
            continue

        row_id = report_row.get("row_id")
        if not isinstance(row_id, str) or row_id.strip() == "":
            decision.reject_if(True, "api_compatibility_matrix_local_heavy_policy_fixture_row_id_invalid")
            continue

        observed_ids.append(row_id)
        expected_row = expected_by_id.get(row_id)
        if expected_row is None:
            decision.reject_if(True, "api_compatibility_matrix_local_heavy_policy_fixture_row_id_invalid")
            continue

        decision.reject_if(
            report_row.get("row_status") != "verified",
            "api_compatibility_matrix_local_heavy_policy_fixture_row_status_mismatch",
        )
        decision.reject_if(
            report_row.get("from_version") != expected_row["from_version"],
            "api_compatibility_matrix_local_heavy_policy_fixture_row_version_pair_mismatch",
        )
        decision.reject_if(
            report_row.get("to_version") != expected_row["to_version"],
            "api_compatibility_matrix_local_heavy_policy_fixture_row_version_pair_mismatch",
        )
        decision.reject_if(
            report_row.get("route_selector") != expected_row["route_selector"],
            "api_compatibility_matrix_local_heavy_policy_fixture_row_route_selector_mismatch",
        )
        decision.reject_if(
            report_row.get("change_class") != expected_row["change_class"],
            "api_compatibility_matrix_local_heavy_policy_fixture_row_change_class_mismatch",
        )

        expected_status = (
            "compatible"
            if expected_row["change_class"] in COMPATIBLE_CHANGE_CLASSES
            else "incompatible"
        )
        decision.reject_if(
            report_row.get("compatibility_status") != expected_status,
            "api_compatibility_matrix_local_heavy_policy_fixture_row_change_class_mismatch",
        )
        decision.reject_if(
            report_row.get("expected_final_decision") != expected_row["expected_final_decision"],
            "api_compatibility_matrix_local_heavy_policy_fixture_row_decision_mismatch",
        )
        decision.reject_if(
            report_row.get("observed_final_decision") != expected_row["expected_final_decision"],
            "api_compatibility_matrix_local_heavy_policy_fixture_row_decision_mismatch",
        )
        decision.reject_if(
            report_row.get("expected_reason_code") != expected_row["expected_reason_code"],
            "api_compatibility_matrix_local_heavy_policy_fixture_row_reason_code_mismatch",
        )
        decision.reject_if(
            report_row.get("observed_reason_code") != expected_row["expected_reason_code"],
            "api_compatibility_matrix_local_heavy_policy_fixture_row_reason_code_mismatch",
        )

    deduped_ids = _dedupe_preserve_order(observed_ids)
    decision.reject_if(
        len(deduped_ids) != len(observed_ids),
        "api_compatibility_matrix_local_heavy_policy_fixture_row_duplicate",
    )
    for expected_id in expected_by_id:
        decision.reject_if(
            expected_id not in observed_ids,
            "api_compatibility_matrix_local_heavy_policy_fixture_row_missing",
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
        "matrix_artifact_status",
        "compatibility_class_projection_status",
        "local_heavy_scope_status",
        "fail_closed_status",
        "performance_budget_status",
        "reason_taxonomy_version",
        "reason_codes_csv",
        "artifact_schema_version",
        "fixture_schema_version",
        "fixture_path",
        "required_row_ids_csv",
        "matrix_rows",
        "matrix_row_count",
        "compatible_row_count",
        "incompatible_row_count",
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
        "api_compatibility_matrix_local_heavy_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("status") not in {"pass", "fail"},
        "api_compatibility_matrix_local_heavy_policy_status_invalid",
    )
    decision.reject_if(
        report.get("final_decision") not in {"GO", "NO-GO"},
        "api_compatibility_matrix_local_heavy_policy_final_decision_invalid",
    )
    decision.reject_if(
        report.get("final_decision") != expected_final_decision,
        "api_compatibility_matrix_local_heavy_policy_final_decision_mismatch",
    )
    decision.reject_if(
        report.get("reason_taxonomy_version") != REASON_TAXONOMY_VERSION,
        "api_compatibility_matrix_local_heavy_policy_schema_mismatch",
    )
    decision.reject_if(
        report.get("reason_codes_csv") != REASON_CODES_CSV,
        "api_compatibility_matrix_local_heavy_policy_schema_mismatch",
    )

    for marker_name in (
        "matrix_artifact_status",
        "compatibility_class_projection_status",
        "local_heavy_scope_status",
        "fail_closed_status",
        "performance_budget_status",
    ):
        decision.reject_if(
            report.get(marker_name) != "verified",
            "api_compatibility_matrix_local_heavy_policy_marker_missing",
        )

    lane_mode = report.get("lane_mode")
    decision.reject_if(
        lane_mode not in {"dry-run", "run"},
        "api_compatibility_matrix_local_heavy_policy_lane_mode_invalid",
    )

    decision.reject_if(
        report.get("artifact_schema_version") != ARTIFACT_SCHEMA,
        "api_compatibility_matrix_local_heavy_policy_artifact_schema_mismatch",
    )
    decision.reject_if(
        report.get("fixture_schema_version") != FIXTURE_SCHEMA,
        "api_compatibility_matrix_local_heavy_policy_fixture_schema_mismatch",
    )
    decision.reject_if(
        report.get("fixture_path")
        != "fixtures/runtime/api_compatibility_matrix_local_heavy_fixture_matrix.txt",
        "api_compatibility_matrix_local_heavy_policy_fixture_schema_mismatch",
    )
    decision.reject_if(
        report.get("required_row_ids_csv") != REQUIRED_ROW_IDS_CSV,
        "api_compatibility_matrix_local_heavy_policy_fixture_schema_mismatch",
    )

    _validate_matrix_rows(report.get("matrix_rows"), fixture_rows, decision)

    matrix_row_count = report.get("matrix_row_count")
    decision.reject_if(
        not _is_non_negative_int(matrix_row_count),
        "api_compatibility_matrix_local_heavy_policy_fixture_rows_invalid",
    )
    if isinstance(matrix_row_count, int):
        decision.reject_if(
            matrix_row_count != len(fixture_rows),
            "api_compatibility_matrix_local_heavy_policy_fixture_row_count_mismatch",
        )

    compatible_row_count = report.get("compatible_row_count")
    incompatible_row_count = report.get("incompatible_row_count")
    decision.reject_if(
        not _is_non_negative_int(compatible_row_count)
        or not _is_non_negative_int(incompatible_row_count),
        "api_compatibility_matrix_local_heavy_policy_fixture_rows_invalid",
    )

    expected_compatible = sum(
        1 for row in fixture_rows if row["change_class"] in COMPATIBLE_CHANGE_CLASSES
    )
    expected_incompatible = sum(
        1 for row in fixture_rows if row["change_class"] in INCOMPATIBLE_REASON_BY_CLASS
    )
    if isinstance(compatible_row_count, int):
        decision.reject_if(
            compatible_row_count != expected_compatible,
            "api_compatibility_matrix_local_heavy_policy_fixture_row_count_mismatch",
        )
    if isinstance(incompatible_row_count, int):
        decision.reject_if(
            incompatible_row_count != expected_incompatible,
            "api_compatibility_matrix_local_heavy_policy_fixture_row_count_mismatch",
        )

    command_count = report.get("command_count")
    decision.reject_if(
        not _is_non_negative_int(command_count),
        "api_compatibility_matrix_local_heavy_policy_command_count_invalid",
    )

    execution_reason_code = report.get("execution_reason_code")
    if lane_mode == "dry-run":
        decision.reject_if(
            execution_reason_code != DRY_RUN_REASON_CODE,
            "api_compatibility_matrix_local_heavy_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            ci_fast_gate != "PASS",
            "ci_fast_gate_failed",
        )
        decision.reject_if(
            command_count != 0,
            "api_compatibility_matrix_local_heavy_policy_command_count_mismatch",
        )
    elif lane_mode == "run":
        decision.reject_if(
            execution_reason_code != RUN_MODE_REASON_CODE,
            "api_compatibility_matrix_local_heavy_policy_execution_reason_code_mismatch",
        )
        decision.reject_if(
            ci_fast_gate != "FAIL",
            "ci_fast_gate_failed",
        )
        decision.reject_if(
            command_count != 0,
            "api_compatibility_matrix_local_heavy_policy_command_count_mismatch",
        )

    elapsed_seconds = report.get("elapsed_seconds")
    decision.reject_if(
        not _is_non_negative_int(elapsed_seconds),
        "api_compatibility_matrix_local_heavy_policy_elapsed_seconds_invalid",
    )
    max_seconds = report.get("max_seconds")
    decision.reject_if(
        not _is_non_negative_int(max_seconds),
        "api_compatibility_matrix_local_heavy_policy_max_seconds_invalid",
    )
    if isinstance(elapsed_seconds, int) and isinstance(max_seconds, int):
        decision.reject_if(
            elapsed_seconds > max_seconds,
            "api_compatibility_matrix_local_heavy_policy_runtime_budget_exceeded",
        )
        decision.reject_if(
            max_seconds > MAX_BUDGET_SECONDS,
            "api_compatibility_matrix_local_heavy_policy_runtime_budget_exceeded",
        )

    final_decision, reason_codes = decision.finalize("none")
    status = "pass" if final_decision == "GO" else "fail"
    policy_status = "verified" if final_decision == "GO" else "rejected"

    policy_report = {
        "schema_version": POLICY_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "api_compatibility_matrix_local_heavy_policy_status": policy_status,
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
    print(f"api_compatibility_matrix_local_heavy_policy_status={policy_status}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes={reason_codes_csv}")
    if output_json is not None:
        print(f"policy_report_file={output_json}")

    if final_decision != "GO":
        fail(f"api compatibility matrix policy rejected: {reason_codes_csv}")
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
        "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_CONTRACT_MAX_SECONDS",
        args.max_seconds,
    )
    _ = require_positive_int(
        "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_CONTRACT_COMMAND_MAX_SECONDS",
        args.command_max_seconds,
    )
    if max_seconds > MAX_BUDGET_SECONDS:
        fail(
            "max-seconds must be <= "
            f"{MAX_BUDGET_SECONDS} for local-heavy compatibility contract lane"
        )

    mode = require_enum("--mode", args.mode.strip(), ("dry-run", "run"))
    ci_fast_gate = require_enum("--ci-fast-gate", args.ci_fast_gate, ("PASS", "FAIL"))

    fixture_file = Path(args.fixture_file).resolve()
    ops_doc = Path(args.ops_doc).resolve()

    start_epoch = int(time.time())
    with tempfile.TemporaryDirectory(prefix="api-compatibility-local-heavy-contract-") as tmp_dir_raw:
        tmp_dir = Path(tmp_dir_raw)
        summary_report = tmp_dir / "api-compatibility-local-heavy-summary.json"
        policy_report = tmp_dir / "api-compatibility-local-heavy-policy.json"
        tampered_report = tmp_dir / "api-compatibility-local-heavy-summary.tampered.json"
        tampered_policy_report = tmp_dir / "api-compatibility-local-heavy-policy.tampered.json"

        lane_output = _invoke_with_captured_output(
            _run_lane,
            argparse.Namespace(
                mode=mode,
                ci_fast_gate=ci_fast_gate,
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
                "matrix_artifact_status=verified",
                "compatibility_class_projection_status=verified",
            ),
            "api compatibility local-heavy lane output",
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
                "api_compatibility_matrix_local_heavy_policy_status=verified",
            ),
            "api compatibility local-heavy policy output",
        )

        tampered_payload = dict(load_json(summary_report))
        tampered_rows = tampered_payload.get("matrix_rows")
        if not isinstance(tampered_rows, list) or not tampered_rows:
            fail("contract tamper setup requires non-empty matrix_rows")
        first_row = tampered_rows[0]
        if not isinstance(first_row, dict):
            fail("contract tamper setup requires dict matrix row")
        first_row["row_status"] = "missing"
        tampered_payload["matrix_rows"] = tampered_rows
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
            fail("expected tampered compatibility matrix report to fail policy checker")
        if TAMPER_REASON_CODE not in str(tampered_error):
            fail(
                "expected deterministic tamper reason marker for local-heavy compatibility "
                f"contract lane: {TAMPER_REASON_CODE}"
            )
        _require_output_markers(
            tampered_output,
            (
                "status=error",
                "final_decision=NO-GO",
                "api_compatibility_matrix_local_heavy_policy_status=rejected",
            ),
            "api compatibility local-heavy tampered policy output",
        )

        _require_doc_markers(
            doc_file=ops_doc,
            required_markers=OPS_REQUIRED_MARKERS,
            reason_code=DOCS_MARKER_REASON_CODE,
        )

        elapsed_seconds = int(time.time()) - start_epoch
        if elapsed_seconds > max_seconds:
            fail(
                "api compatibility local-heavy contract lane exceeded runtime budget: "
                f"{elapsed_seconds}s (max={max_seconds}s)"
            )

        policy_payload = load_json(policy_report)
        lane_report = {
            "schema_version": CONTRACT_LANE_SCHEMA,
            "status": "pass",
            "final_decision": "GO",
            "api_compatibility_matrix_local_heavy_contract_status": "verified",
            "api_compatibility_matrix_local_heavy_policy_status": policy_payload.get(
                "api_compatibility_matrix_local_heavy_policy_status",
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
    print("api_compatibility_matrix_local_heavy_contract_status=verified")
    print("api_compatibility_matrix_local_heavy_policy_status=verified")
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
        description="API compatibility matrix local-heavy lane and policy contracts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    run_lane_parser = subparsers.add_parser(
        "run-lane",
        help="Execute API compatibility matrix local-heavy lane.",
    )
    run_lane_parser.add_argument(
        "--mode",
        default=os.environ.get("KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_MODE", "dry-run"),
        help="Lane mode: dry-run|run.",
    )
    run_lane_parser.add_argument(
        "--ci-fast-gate",
        default=os.environ.get("KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_CI_FAST_GATE", "PASS"),
        help="CI fast-gate mode: PASS for dry-run, FAIL for run.",
    )
    run_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_MAX_SECONDS",
            DEFAULT_MAX_SECONDS,
        ),
        help="Maximum lane runtime budget in seconds.",
    )
    run_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_COMMAND_MAX_SECONDS",
            DEFAULT_COMMAND_MAX_SECONDS,
        ),
        help="Maximum runtime budget for nested commands in run mode.",
    )
    run_lane_parser.add_argument(
        "--fixture-file",
        default=str(FIXTURE_PATH),
        help="Fixture matrix file for compatibility matrix checks.",
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
        help="Validate API compatibility matrix local-heavy policy.",
    )
    check_policy_parser.add_argument("--report-file", required=True)
    check_policy_parser.add_argument("--fixture-file", default=str(FIXTURE_PATH))
    check_policy_parser.add_argument("--expected-final-decision", default="GO")
    check_policy_parser.add_argument("--ci-fast-gate", default="PASS")
    check_policy_parser.add_argument("--output-json", default="")
    check_policy_parser.set_defaults(handler=_check_policy)

    contract_lane_parser = subparsers.add_parser(
        "run-contract-lane",
        help="Run API compatibility matrix local-heavy contract lane.",
    )
    contract_lane_parser.add_argument(
        "--mode",
        default=os.environ.get(
            "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_CONTRACT_MODE",
            "dry-run",
        ),
        help="Contract lane mode: dry-run|run.",
    )
    contract_lane_parser.add_argument(
        "--ci-fast-gate",
        default=os.environ.get(
            "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_CONTRACT_CI_FAST_GATE",
            "PASS",
        ),
        help="PASS for dry-run, FAIL for run mode.",
    )
    contract_lane_parser.add_argument(
        "--max-seconds",
        default=os.environ.get(
            "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_CONTRACT_MAX_SECONDS",
            DEFAULT_MAX_SECONDS,
        ),
        help="Maximum contract-lane runtime budget in seconds.",
    )
    contract_lane_parser.add_argument(
        "--command-max-seconds",
        default=os.environ.get(
            "KAMN_API_COMPATIBILITY_MATRIX_LOCAL_HEAVY_CONTRACT_COMMAND_MAX_SECONDS",
            DEFAULT_COMMAND_MAX_SECONDS,
        ),
        help="Maximum runtime budget for nested commands in run mode.",
    )
    contract_lane_parser.add_argument("--fixture-file", default=str(FIXTURE_PATH))
    contract_lane_parser.add_argument("--ops-doc", default=str(OPS_DOC_PATH))
    contract_lane_parser.add_argument("--output-json", default="")
    contract_lane_parser.add_argument("--policy-output-json", default="")
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
    if hasattr(args, "ci_fast_gate"):
        args.ci_fast_gate = args.ci_fast_gate.strip()
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
