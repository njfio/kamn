#!/usr/bin/env python3
"""Generate and validate deterministic Kolme wrapper-inventory baselines."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
import subprocess
import sys
from typing import Any

BASELINE_SCHEMA_VERSION = "kamn.kolme.wrapper-inventory-baseline.v1"
DELTA_REPORT_SCHEMA_VERSION = "kamn.kolme.wrapper-inventory-delta-report.v1"
MATRIX_SCHEMA_VERSION = "kamn.kolme.lane-migration-matrix.v1"
TREND_THRESHOLD_SCHEMA_VERSION = "kamn.kolme.wrapper-budget-trend-thresholds.v1"


class PolicyValidationError(Exception):
    """Structured policy-validation failure carrying a deterministic reason code."""

    def __init__(self, reason_code: str, message: str) -> None:
        super().__init__(message)
        self.reason_code = reason_code
        self.message = message


def fail(message: str) -> None:
    raise PolicyValidationError("policy_validation_failed", message)


def fail_with_reason(reason_code: str, message: str) -> None:
    raise PolicyValidationError(reason_code, message)


def load_json_object(path: Path, *, label: str) -> dict[str, Any]:
    if not path.is_file():
        reason_code = "input_file_not_found"
        if label == "wrapper inventory baseline":
            reason_code = "baseline_file_not_found"
        elif label == "wrapper budget trend thresholds":
            reason_code = "trend_threshold_file_not_found"
        elif label == "lane migration matrix":
            reason_code = "lane_matrix_file_not_found"
        fail_with_reason(reason_code, f"{label} not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        reason_code = "input_json_invalid"
        if label == "wrapper inventory baseline":
            reason_code = "baseline_json_invalid"
        elif label == "wrapper budget trend thresholds":
            reason_code = "trend_threshold_json_invalid"
        elif label == "lane migration matrix":
            reason_code = "lane_matrix_json_invalid"
        fail_with_reason(reason_code, f"invalid JSON in {label} {path}: {exc}")
    if not isinstance(payload, dict):
        reason_code = "input_payload_not_object"
        if label == "wrapper inventory baseline":
            reason_code = "baseline_payload_not_object"
        elif label == "wrapper budget trend thresholds":
            reason_code = "trend_threshold_payload_not_object"
        elif label == "lane migration matrix":
            reason_code = "lane_matrix_payload_not_object"
        fail_with_reason(reason_code, f"expected JSON object for {label}: {path}")
    return payload


def require_non_empty_string(payload: dict[str, Any], key: str, *, label: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        fail(f"{label} {key} must be a non-empty string")
    return value.strip()


def require_int(payload: dict[str, Any], key: str, *, label: str) -> int:
    value = payload.get(key)
    if not isinstance(value, int):
        fail(f"{label} {key} must be an integer")
    return value


def to_repo_relative(path: Path, repo_root: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return str(path)


def count_shell_loc(path: Path) -> int:
    if path.is_symlink():
        return 1
    try:
        with path.open("r", encoding="utf-8") as handle:
            return sum(1 for _ in handle)
    except OSError as exc:
        fail(f"failed to read wrapper for shell LOC accounting {path}: {exc}")


def resolve_manifest_file_from_dispatchers(*, repo_root: Path, source_entry: str) -> str:
    wrapper_name = Path(source_entry).name
    dispatchers = (
        repo_root / "scripts/kolme/run_contract_lane_dispatch.sh",
        repo_root / "scripts/kolme/run_lane_dispatch.sh",
    )

    resolution_errors: list[str] = []
    for dispatcher in dispatchers:
        if not dispatcher.is_file():
            continue

        result = subprocess.run(
            [
                "bash",
                str(dispatcher),
                "--lane-wrapper",
                wrapper_name,
                "--resolve-manifest-path",
            ],
            capture_output=True,
            text=True,
            check=False,
        )
        if result.returncode != 0:
            stderr = result.stderr.strip() or "unknown dispatcher error"
            resolution_errors.append(f"{dispatcher.name}: {stderr}")
            continue

        resolved = result.stdout.strip()
        if not resolved:
            resolution_errors.append(f"{dispatcher.name}: returned empty manifest path")
            continue

        manifest_path = Path(resolved)
        if not manifest_path.is_absolute():
            manifest_path = (repo_root / manifest_path).resolve()
        if not manifest_path.is_file():
            resolution_errors.append(
                f"{dispatcher.name}: resolved manifest does not exist {manifest_path}"
            )
            continue

        return to_repo_relative(manifest_path, repo_root)

    if not resolution_errors:
        fail(
            "failed to resolve manifest for wrapper "
            f"{wrapper_name} from source_entry {source_entry}: no dispatcher script found"
        )

    fail(
        "failed to resolve manifest for wrapper "
        f"{wrapper_name} from source_entry {source_entry}: "
        + " | ".join(resolution_errors)
    )


def build_inventory(*, matrix_file: Path, repo_root: Path) -> dict[str, Any]:
    payload = load_json_object(matrix_file, label="lane migration matrix")
    if payload.get("schema_version") != MATRIX_SCHEMA_VERSION:
        fail(
            "lane migration matrix schema_version must be "
            f"{MATRIX_SCHEMA_VERSION}"
        )

    lanes = payload.get("lanes")
    if not isinstance(lanes, list) or not lanes:
        fail("lane migration matrix lanes must be a non-empty array")

    inventory_lanes: list[dict[str, Any]] = []
    lane_ids_seen: set[str] = set()
    symlink_wrapper_count = 0
    regular_file_wrapper_count = 0
    total_shell_loc = 0

    for index, lane in enumerate(sorted(lanes, key=lambda item: str(item.get("lane_id", "")))):
        if not isinstance(lane, dict):
            fail(f"lane migration matrix lane[{index}] must be an object")

        lane_id = require_non_empty_string(lane, "lane_id", label=f"lane[{index}]")
        if lane_id in lane_ids_seen:
            fail(f"lane migration matrix lane_id must be unique, found duplicate: {lane_id}")
        lane_ids_seen.add(lane_id)

        source_entry = require_non_empty_string(
            lane,
            "source_entry",
            label=f"lane[{index}]",
        )
        priority = require_non_empty_string(lane, "priority", label=f"lane[{index}]")
        status = require_non_empty_string(lane, "status", label=f"lane[{index}]")
        target_runner = require_non_empty_string(
            lane,
            "target_runner",
            label=f"lane[{index}]",
        )

        wrapper_path = repo_root / source_entry
        if not wrapper_path.exists():
            fail(f"lane wrapper path does not exist for {lane_id}: {source_entry}")
        if not wrapper_path.is_file():
            fail(f"lane wrapper path is not a file for {lane_id}: {source_entry}")

        wrapper_kind = "symlink" if wrapper_path.is_symlink() else "regular_file"
        if wrapper_kind == "symlink":
            symlink_wrapper_count += 1
        else:
            regular_file_wrapper_count += 1

        shell_loc = count_shell_loc(wrapper_path)
        if shell_loc < 1:
            fail(f"shell LOC must be >= 1 for lane {lane_id}")
        total_shell_loc += shell_loc

        manifest_file_override = lane.get("manifest_file")
        if manifest_file_override is None:
            manifest_file = resolve_manifest_file_from_dispatchers(
                repo_root=repo_root,
                source_entry=source_entry,
            )
        else:
            if (
                not isinstance(manifest_file_override, str)
                or not manifest_file_override.strip()
            ):
                fail(
                    f"lane[{index}] manifest_file override must be a non-empty string when provided"
                )
            manifest_path = Path(manifest_file_override.strip())
            if not manifest_path.is_absolute():
                manifest_path = (repo_root / manifest_path).resolve()
            if not manifest_path.is_file():
                fail(
                    f"lane manifest override does not exist for {lane_id}: {manifest_file_override}"
                )
            manifest_file = to_repo_relative(manifest_path, repo_root)

        inventory_lanes.append(
            {
                "lane_id": lane_id,
                "priority": priority,
                "status": status,
                "source_entry": source_entry,
                "manifest_file": manifest_file,
                "target_runner": target_runner,
                "wrapper_kind": wrapper_kind,
                "shell_loc": shell_loc,
            }
        )

    return {
        "schema_version": BASELINE_SCHEMA_VERSION,
        "source_matrix_file": to_repo_relative(matrix_file, repo_root),
        "wrapper_count": len(inventory_lanes),
        "symlink_wrapper_count": symlink_wrapper_count,
        "regular_file_wrapper_count": regular_file_wrapper_count,
        "total_shell_loc": total_shell_loc,
        "lanes": inventory_lanes,
    }


def validate_baseline_payload(payload: dict[str, Any]) -> None:
    if payload.get("schema_version") != BASELINE_SCHEMA_VERSION:
        fail_with_reason(
            "baseline_schema_mismatch",
            "baseline schema_version must be "
            f"{BASELINE_SCHEMA_VERSION}"
        )

    lanes = payload.get("lanes")
    if not isinstance(lanes, list) or not lanes:
        fail_with_reason("baseline_lanes_invalid", "baseline lanes must be a non-empty array")

    lane_ids_seen: set[str] = set()
    for index, lane in enumerate(lanes):
        if not isinstance(lane, dict):
            fail_with_reason("baseline_lane_object_invalid", f"baseline lane[{index}] must be an object")
        lane_id = require_non_empty_string(lane, "lane_id", label=f"baseline lane[{index}]")
        if lane_id in lane_ids_seen:
            fail_with_reason(
                "baseline_lane_id_duplicate",
                f"baseline lane_id must be unique, found duplicate: {lane_id}",
            )
        lane_ids_seen.add(lane_id)
        require_non_empty_string(lane, "source_entry", label=f"baseline lane[{index}]")
        require_non_empty_string(lane, "manifest_file", label=f"baseline lane[{index}]")
        require_non_empty_string(lane, "wrapper_kind", label=f"baseline lane[{index}]")
        shell_loc = require_int(lane, "shell_loc", label=f"baseline lane[{index}]")
        if shell_loc < 1:
            fail_with_reason("baseline_lane_shell_loc_invalid", f"baseline lane[{index}] shell_loc must be >= 1")

    for key in (
        "wrapper_count",
        "symlink_wrapper_count",
        "regular_file_wrapper_count",
        "total_shell_loc",
    ):
        try:
            value = require_int(payload, key, label="baseline")
        except PolicyValidationError as error:
            if key == "wrapper_count":
                fail_with_reason("baseline_wrapper_count_invalid", error.message)
            if key == "symlink_wrapper_count":
                fail_with_reason("baseline_symlink_wrapper_count_invalid", error.message)
            if key == "regular_file_wrapper_count":
                fail_with_reason("baseline_regular_file_wrapper_count_invalid", error.message)
            if key == "total_shell_loc":
                fail_with_reason("baseline_total_shell_loc_invalid", error.message)
            raise
        if value < 0:
            if key == "wrapper_count":
                fail_with_reason("baseline_wrapper_count_invalid", f"baseline {key} must be >= 0")
            if key == "symlink_wrapper_count":
                fail_with_reason("baseline_symlink_wrapper_count_invalid", f"baseline {key} must be >= 0")
            if key == "regular_file_wrapper_count":
                fail_with_reason("baseline_regular_file_wrapper_count_invalid", f"baseline {key} must be >= 0")
            if key == "total_shell_loc":
                fail_with_reason("baseline_total_shell_loc_invalid", f"baseline {key} must be >= 0")
            fail_with_reason("baseline_totals_invalid", f"baseline {key} must be >= 0")


def load_trend_thresholds(path: Path) -> dict[str, Any]:
    payload = load_json_object(path, label="wrapper budget trend thresholds")
    if payload.get("schema_version") != TREND_THRESHOLD_SCHEMA_VERSION:
        fail_with_reason(
            "trend_threshold_schema_mismatch",
            "wrapper budget trend threshold schema_version must be "
            f"{TREND_THRESHOLD_SCHEMA_VERSION}"
        )

    try:
        max_wrapper_count_increase = require_int(
            payload,
            "max_wrapper_count_increase",
            label="wrapper budget trend thresholds",
        )
    except PolicyValidationError as error:
        fail_with_reason("trend_threshold_wrapper_count_invalid", error.message)
    if max_wrapper_count_increase < 0:
        fail_with_reason(
            "trend_threshold_wrapper_count_invalid",
            "wrapper budget trend max_wrapper_count_increase must be >= 0",
        )

    try:
        max_total_shell_loc_increase = require_int(
            payload,
            "max_total_shell_loc_increase",
            label="wrapper budget trend thresholds",
        )
    except PolicyValidationError as error:
        fail_with_reason("trend_threshold_total_shell_loc_invalid", error.message)
    if max_total_shell_loc_increase < 0:
        fail_with_reason(
            "trend_threshold_total_shell_loc_invalid",
            "wrapper budget trend max_total_shell_loc_increase must be >= 0",
        )

    enforce_lane_shell_loc_nonincreasing = payload.get(
        "enforce_lane_shell_loc_nonincreasing"
    )
    if not isinstance(enforce_lane_shell_loc_nonincreasing, bool):
        fail_with_reason(
            "trend_threshold_lane_nonincreasing_invalid",
            "wrapper budget trend thresholds enforce_lane_shell_loc_nonincreasing "
            "must be a boolean"
        )

    try:
        min_wrapper_count_reduction = require_int(
            payload,
            "min_wrapper_count_reduction",
            label="wrapper budget trend thresholds",
        )
    except PolicyValidationError as error:
        fail_with_reason("trend_threshold_min_wrapper_reduction_invalid", error.message)
    if min_wrapper_count_reduction < 0:
        fail_with_reason(
            "trend_threshold_min_wrapper_reduction_invalid",
            "wrapper budget trend min_wrapper_count_reduction must be >= 0",
        )

    try:
        min_total_shell_loc_reduction = require_int(
            payload,
            "min_total_shell_loc_reduction",
            label="wrapper budget trend thresholds",
        )
    except PolicyValidationError as error:
        fail_with_reason("trend_threshold_min_total_shell_loc_reduction_invalid", error.message)
    if min_total_shell_loc_reduction < 0:
        fail_with_reason(
            "trend_threshold_min_total_shell_loc_reduction_invalid",
            "wrapper budget trend min_total_shell_loc_reduction must be >= 0",
        )

    return {
        "max_wrapper_count_increase": max_wrapper_count_increase,
        "max_total_shell_loc_increase": max_total_shell_loc_increase,
        "enforce_lane_shell_loc_nonincreasing": enforce_lane_shell_loc_nonincreasing,
        "min_wrapper_count_reduction": min_wrapper_count_reduction,
        "min_total_shell_loc_reduction": min_total_shell_loc_reduction,
    }


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def command_generate(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    matrix_file = Path(args.matrix_file).resolve()
    output_json = Path(args.output_json).resolve()

    inventory = build_inventory(matrix_file=matrix_file, repo_root=repo_root)
    write_json(output_json, inventory)

    print("status=generated")
    print(f"wrapper_count={inventory['wrapper_count']}")
    print(f"total_shell_loc={inventory['total_shell_loc']}")
    print(f"output_json={output_json}")
    return 0


def index_lanes(payload: dict[str, Any], *, label: str) -> dict[str, dict[str, Any]]:
    lanes = payload["lanes"]
    indexed: dict[str, dict[str, Any]] = {}
    for lane in lanes:
        lane_id = lane["lane_id"]
        if lane_id in indexed:
            fail(f"duplicate {label} lane_id while indexing: {lane_id}")
        indexed[lane_id] = lane
    return indexed


def command_check(args: argparse.Namespace) -> int:
    repo_root = Path(args.repo_root).resolve()
    matrix_file = Path(args.matrix_file).resolve()
    baseline_file = Path(args.baseline_file).resolve()
    trend_mode = bool(args.trend_mode)
    max_wrapper_count_increase = int(args.max_wrapper_count_increase)
    max_total_shell_loc_increase = int(args.max_total_shell_loc_increase)
    min_wrapper_count_reduction = int(args.min_wrapper_count_reduction)
    min_total_shell_loc_reduction = int(args.min_total_shell_loc_reduction)
    enforce_lane_shell_loc_nonincreasing = True

    try:
        if max_wrapper_count_increase < 0:
            fail_with_reason("trend_threshold_wrapper_count_invalid", "--max-wrapper-count-increase must be >= 0")
        if max_total_shell_loc_increase < 0:
            fail_with_reason(
                "trend_threshold_total_shell_loc_invalid",
                "--max-total-shell-loc-increase must be >= 0",
            )
        if min_wrapper_count_reduction < 0:
            fail_with_reason(
                "trend_threshold_min_wrapper_reduction_invalid",
                "--min-wrapper-count-reduction must be >= 0",
            )
        if min_total_shell_loc_reduction < 0:
            fail_with_reason(
                "trend_threshold_min_total_shell_loc_reduction_invalid",
                "--min-total-shell-loc-reduction must be >= 0",
            )
        if args.threshold_file and not trend_mode:
            fail_with_reason(
                "trend_mode_required_for_threshold_file",
                "--threshold-file requires --trend-mode",
            )

        if args.threshold_file:
            threshold_file = Path(args.threshold_file).resolve()
            thresholds = load_trend_thresholds(threshold_file)
            max_wrapper_count_increase = int(thresholds["max_wrapper_count_increase"])
            max_total_shell_loc_increase = int(thresholds["max_total_shell_loc_increase"])
            enforce_lane_shell_loc_nonincreasing = bool(
                thresholds["enforce_lane_shell_loc_nonincreasing"]
            )
            min_wrapper_count_reduction = int(thresholds["min_wrapper_count_reduction"])
            min_total_shell_loc_reduction = int(thresholds["min_total_shell_loc_reduction"])

        baseline = load_json_object(baseline_file, label="wrapper inventory baseline")
        validate_baseline_payload(baseline)

        current = build_inventory(matrix_file=matrix_file, repo_root=repo_root)
    except PolicyValidationError as error:
        print("status=fail")
        print(f"mode={'trend' if trend_mode else 'strict'}")
        print("wrapper_count_delta=0")
        print("total_shell_loc_delta=0")
        print("violation_count=1")
        print(f"reason_codes={error.reason_code}")
        print(f"error={error.message}")
        return 1

    baseline_lanes = index_lanes(baseline, label="baseline")
    current_lanes = index_lanes(current, label="current")

    baseline_lane_ids = set(baseline_lanes)
    current_lane_ids = set(current_lanes)

    violations: list[str] = []
    reason_codes: set[str] = set()

    missing_lanes = sorted(baseline_lane_ids - current_lane_ids)
    extra_lanes = sorted(current_lane_ids - baseline_lane_ids)
    if missing_lanes:
        violations.append(f"missing lanes in current inventory: {', '.join(missing_lanes)}")
        reason_codes.add("missing_lanes_in_current_inventory")
    if extra_lanes:
        violations.append(f"unexpected new lanes in current inventory: {', '.join(extra_lanes)}")
        reason_codes.add("unexpected_new_lanes_in_current_inventory")

    lane_deltas: list[dict[str, Any]] = []
    for lane_id in sorted(baseline_lane_ids & current_lane_ids):
        baseline_lane = baseline_lanes[lane_id]
        current_lane = current_lanes[lane_id]
        shell_loc_delta = int(current_lane["shell_loc"]) - int(baseline_lane["shell_loc"])
        lane_deltas.append(
            {
                "lane_id": lane_id,
                "baseline_shell_loc": baseline_lane["shell_loc"],
                "current_shell_loc": current_lane["shell_loc"],
                "shell_loc_delta": shell_loc_delta,
                "baseline_wrapper_kind": baseline_lane["wrapper_kind"],
                "current_wrapper_kind": current_lane["wrapper_kind"],
                "baseline_manifest_file": baseline_lane["manifest_file"],
                "current_manifest_file": current_lane["manifest_file"],
            }
        )

        for key in ("source_entry", "manifest_file", "wrapper_kind"):
            if baseline_lane[key] != current_lane[key]:
                violations.append(
                    f"lane {lane_id} changed {key}: baseline={baseline_lane[key]} current={current_lane[key]}"
                )
                if key == "source_entry":
                    reason_codes.add("lane_source_entry_drift")
                elif key == "manifest_file":
                    reason_codes.add("lane_manifest_file_drift")
                elif key == "wrapper_kind":
                    reason_codes.add("lane_wrapper_kind_drift")
        if trend_mode:
            if enforce_lane_shell_loc_nonincreasing and shell_loc_delta > 0:
                violations.append(
                    "lane "
                    f"{lane_id} shell_loc increased beyond nonincreasing policy: "
                    f"baseline={baseline_lane['shell_loc']} current={current_lane['shell_loc']}"
                )
                reason_codes.add("lane_shell_loc_increase_violation")
        elif shell_loc_delta != 0:
            violations.append(
                f"lane {lane_id} shell_loc drifted: baseline={baseline_lane['shell_loc']} current={current_lane['shell_loc']}"
            )
            reason_codes.add("lane_shell_loc_drift")

    totals_delta = {
        "wrapper_count_delta": current["wrapper_count"] - baseline["wrapper_count"],
        "symlink_wrapper_count_delta": current["symlink_wrapper_count"] - baseline["symlink_wrapper_count"],
        "regular_file_wrapper_count_delta": current["regular_file_wrapper_count"] - baseline["regular_file_wrapper_count"],
        "total_shell_loc_delta": current["total_shell_loc"] - baseline["total_shell_loc"],
    }

    if trend_mode:
        if totals_delta["wrapper_count_delta"] > max_wrapper_count_increase:
            violations.append(
                "wrapper_count_delta exceeded trend threshold: "
                f"delta={totals_delta['wrapper_count_delta']} "
                f"threshold={max_wrapper_count_increase}"
            )
            reason_codes.add("wrapper_count_delta_threshold_exceeded")
        if totals_delta["total_shell_loc_delta"] > max_total_shell_loc_increase:
            violations.append(
                "total_shell_loc_delta exceeded trend threshold: "
                f"delta={totals_delta['total_shell_loc_delta']} "
                f"threshold={max_total_shell_loc_increase}"
            )
            reason_codes.add("total_shell_loc_delta_threshold_exceeded")
        if (
            min_wrapper_count_reduction > 0
            and totals_delta["wrapper_count_delta"] > -min_wrapper_count_reduction
        ):
            violations.append(
                "wrapper_count reduction target unmet: "
                f"delta={totals_delta['wrapper_count_delta']} "
                f"target_reduction={min_wrapper_count_reduction}"
            )
            reason_codes.add("wrapper_count_reduction_target_unmet")
        if (
            min_total_shell_loc_reduction > 0
            and totals_delta["total_shell_loc_delta"] > -min_total_shell_loc_reduction
        ):
            violations.append(
                "total_shell_loc reduction target unmet: "
                f"delta={totals_delta['total_shell_loc_delta']} "
                f"target_reduction={min_total_shell_loc_reduction}"
            )
            reason_codes.add("total_shell_loc_reduction_target_unmet")

    sorted_reason_codes = sorted(reason_codes)

    report_payload = {
        "schema_version": DELTA_REPORT_SCHEMA_VERSION,
        "baseline_file": to_repo_relative(baseline_file, repo_root),
        "current_matrix_file": to_repo_relative(matrix_file, repo_root),
        "baseline_totals": {
            "wrapper_count": baseline["wrapper_count"],
            "symlink_wrapper_count": baseline["symlink_wrapper_count"],
            "regular_file_wrapper_count": baseline["regular_file_wrapper_count"],
            "total_shell_loc": baseline["total_shell_loc"],
        },
        "current_totals": {
            "wrapper_count": current["wrapper_count"],
            "symlink_wrapper_count": current["symlink_wrapper_count"],
            "regular_file_wrapper_count": current["regular_file_wrapper_count"],
            "total_shell_loc": current["total_shell_loc"],
        },
        "totals_delta": totals_delta,
        "policy": {
            "mode": "trend" if trend_mode else "strict",
            "max_wrapper_count_increase": max_wrapper_count_increase,
            "max_total_shell_loc_increase": max_total_shell_loc_increase,
            "min_wrapper_count_reduction": min_wrapper_count_reduction,
            "min_total_shell_loc_reduction": min_total_shell_loc_reduction,
            "enforce_lane_shell_loc_nonincreasing": enforce_lane_shell_loc_nonincreasing,
        },
        "lane_deltas": lane_deltas,
        "reason_codes": sorted_reason_codes,
        "violations": violations,
        "status": "fail" if violations else "pass",
    }

    if args.output_json:
        write_json(Path(args.output_json).resolve(), report_payload)

    print(f"status={report_payload['status']}")
    print(f"mode={report_payload['policy']['mode']}")
    print(f"wrapper_count_delta={totals_delta['wrapper_count_delta']}")
    print(f"total_shell_loc_delta={totals_delta['total_shell_loc_delta']}")
    print(f"violation_count={len(violations)}")
    print(f"reason_codes={'none' if not sorted_reason_codes else ','.join(sorted_reason_codes)}")

    if violations:
        for violation in violations:
            print(f"violation={violation}")
        return 1

    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Generate and validate Kolme wrapper inventory baselines.",
    )
    parser.add_argument(
        "--repo-root",
        default=str(Path(__file__).resolve().parents[2]),
        help="Repository root path.",
    )

    subparsers = parser.add_subparsers(dest="command", required=True)

    generate_parser = subparsers.add_parser("generate", help="Generate baseline artifact.")
    generate_parser.add_argument("--matrix-file", required=True, help="Path to lane migration matrix JSON.")
    generate_parser.add_argument("--output-json", required=True, help="Output path for generated baseline JSON.")

    check_parser = subparsers.add_parser("check", help="Check current inventory against baseline.")
    check_parser.add_argument("--matrix-file", required=True, help="Path to lane migration matrix JSON.")
    check_parser.add_argument("--baseline-file", required=True, help="Path to committed baseline JSON.")
    check_parser.add_argument(
        "--trend-mode",
        action="store_true",
        help="Enable trend policy mode (allow reductions, fail only growth above thresholds).",
    )
    check_parser.add_argument(
        "--threshold-file",
        help="Optional trend threshold config JSON (requires --trend-mode).",
    )
    check_parser.add_argument(
        "--max-wrapper-count-increase",
        type=int,
        default=0,
        help="Allowed positive wrapper_count_delta in trend mode.",
    )
    check_parser.add_argument(
        "--max-total-shell-loc-increase",
        type=int,
        default=0,
        help="Allowed positive total_shell_loc_delta in trend mode.",
    )
    check_parser.add_argument(
        "--min-wrapper-count-reduction",
        type=int,
        default=0,
        help="Required minimum wrapper-count reduction in trend mode.",
    )
    check_parser.add_argument(
        "--min-total-shell-loc-reduction",
        type=int,
        default=0,
        help="Required minimum total shell LOC reduction in trend mode.",
    )
    check_parser.add_argument(
        "--output-json",
        help="Optional output path for delta report JSON.",
    )

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()

    if args.command == "generate":
        return command_generate(args)
    if args.command == "check":
        return command_check(args)

    fail_with_reason("unsupported_command", f"unsupported command: {args.command}")
    return 1


if __name__ == "__main__":
    sys.exit(main())
