#!/usr/bin/env python3
"""Check service API tranche-2 wrapper family parity and retirement invariants."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any

MATRIX_SCHEMA = "kamn.runtime.service-api-tranche2-wrapper-family-matrix.v1"
REPORT_SCHEMA = "kamn.runtime.service-api-tranche2-wrapper-family-parity-report.v1"
REASON_TAXONOMY_VERSION = (
    "kamn.runtime.service-api-tranche2-wrapper-family-parity-reason-taxonomy.v1"
)
REASON_CODES_CSV = ",".join(
    [
        "impl_contract_status_marker_missing",
        "impl_missing",
        "impl_not_executable",
        "impl_policy_checker_marker_missing",
        "impl_policy_status_marker_missing",
        "impl_runner_entry_marker_missing",
        "impl_runner_source_marker_missing",
        "impl_tamper_reason_marker_missing",
        "impl_validation_script_marker_missing",
        "matrix_wrapper_entry_invalid",
        "service_api_tranche2_impl_shell_loc_budget_exceeded",
        "service_api_tranche2_wrapper_shell_loc_budget_exceeded",
        "wrapper_dispatch_target_mismatch",
        "wrapper_missing",
        "wrapper_not_symlink",
    ]
)


def _load_json(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except FileNotFoundError as exc:
        raise SystemExit(f"matrix_file_missing:{path}") from exc
    except json.JSONDecodeError as exc:
        raise SystemExit(f"matrix_file_invalid_json:{path}:{exc}") from exc


def _read_text(path: Path) -> str:
    try:
        return path.read_text(encoding="utf-8")
    except OSError as exc:
        raise SystemExit(f"wrapper_read_failed:{path}:{exc}") from exc


def _write_report(output_json: str | None, report: dict[str, Any]) -> None:
    if not output_json:
        return
    path = Path(output_json)
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    except OSError as exc:
        raise SystemExit(f"output_json_write_failed:{path}:{exc}") from exc


def _is_executable(path: Path) -> bool:
    return path.exists() and path.is_file() and os.access(path, os.X_OK)


def _ensure_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"matrix_field_invalid:{key}")
    return value


def _count_shell_lines(path: Path) -> int:
    try:
        with path.open("r", encoding="utf-8") as handle:
            return sum(1 for _ in handle)
    except OSError as exc:
        raise SystemExit(f"wrapper_shell_loc_read_failed:{path}:{exc}") from exc


def _check_wrapper(
    root: Path,
    wrapper_cfg: dict[str, Any],
    expected_dispatch_target: str,
    reason_codes: list[str],
) -> tuple[int, int]:
    wrapper = _ensure_string(wrapper_cfg, "wrapper")
    impl_script = _ensure_string(wrapper_cfg, "impl_script")
    validation_script = _ensure_string(wrapper_cfg, "validation_script")
    policy_checker = _ensure_string(wrapper_cfg, "policy_checker")
    contract_status_key = _ensure_string(wrapper_cfg, "contract_status_key")
    policy_status_key = _ensure_string(wrapper_cfg, "policy_status_key")
    tamper_reason_code = _ensure_string(wrapper_cfg, "tamper_reason_code")

    wrapper_path = root / wrapper
    impl_path = root / impl_script

    wrapper_shell_loc = 0
    impl_shell_loc = 0

    if not wrapper_path.exists():
        reason_codes.append(f"wrapper_missing:{wrapper}")
        return wrapper_shell_loc, impl_shell_loc
    if not wrapper_path.is_symlink():
        reason_codes.append(f"wrapper_not_symlink:{wrapper}")
        return wrapper_shell_loc, impl_shell_loc

    dispatch_target = os.readlink(wrapper_path)
    if dispatch_target != expected_dispatch_target:
        reason_codes.append(f"wrapper_dispatch_target_mismatch:{wrapper}")

    # Symlink wrappers count as one line in wrapper budget accounting.
    wrapper_shell_loc = 1

    if not impl_path.exists():
        reason_codes.append(f"impl_missing:{impl_script}")
        return wrapper_shell_loc, impl_shell_loc
    if not _is_executable(impl_path):
        reason_codes.append(f"impl_not_executable:{impl_script}")
        return wrapper_shell_loc, impl_shell_loc

    impl_text = _read_text(impl_path)
    if 'source "$ROOT_DIR/scripts/runtime/service_api_contract_lane_runner.sh"' not in impl_text:
        reason_codes.append(f"impl_runner_source_marker_missing:{impl_script}")
    if 'service_api_contract_lane_run "$@"' not in impl_text:
        reason_codes.append(f"impl_runner_entry_marker_missing:{impl_script}")
    if f'VALIDATION_SCRIPT="$ROOT_DIR/{validation_script}"' not in impl_text:
        reason_codes.append(f"impl_validation_script_marker_missing:{impl_script}")
    if f'POLICY_CHECKER="$ROOT_DIR/{policy_checker}"' not in impl_text:
        reason_codes.append(f"impl_policy_checker_marker_missing:{impl_script}")
    if f'CONTRACT_STATUS_KEY="{contract_status_key}"' not in impl_text:
        reason_codes.append(f"impl_contract_status_marker_missing:{impl_script}")
    if f'POLICY_STATUS_KEY="{policy_status_key}"' not in impl_text:
        reason_codes.append(f"impl_policy_status_marker_missing:{impl_script}")
    if f'TAMPER_REASON_CODE="{tamper_reason_code}"' not in impl_text:
        reason_codes.append(f"impl_tamper_reason_marker_missing:{impl_script}")

    impl_shell_loc = _count_shell_lines(impl_path)
    return wrapper_shell_loc, impl_shell_loc


def _run(args: argparse.Namespace) -> int:
    root = Path(args.root_dir).resolve()
    matrix_file = Path(args.matrix_file).resolve()
    payload = _load_json(matrix_file)

    if payload.get("schema_version") != MATRIX_SCHEMA:
        raise SystemExit("matrix_schema_mismatch")

    wrappers = payload.get("wrappers")
    if not isinstance(wrappers, list) or not wrappers:
        raise SystemExit("matrix_wrappers_missing")

    expected_dispatch_target = payload.get("expected_dispatch_target")
    if not isinstance(expected_dispatch_target, str) or not expected_dispatch_target:
        raise SystemExit("matrix_expected_dispatch_target_invalid")

    max_wrapper_shell_loc = payload.get("max_wrapper_shell_loc")
    if not isinstance(max_wrapper_shell_loc, int) or max_wrapper_shell_loc < 1:
        raise SystemExit("matrix_max_wrapper_shell_loc_invalid")

    max_impl_shell_loc = payload.get("max_impl_shell_loc")
    if not isinstance(max_impl_shell_loc, int) or max_impl_shell_loc < 1:
        raise SystemExit("matrix_max_impl_shell_loc_invalid")

    reason_codes: list[str] = []
    total_wrapper_shell_loc = 0
    total_impl_shell_loc = 0
    for entry in wrappers:
        if not isinstance(entry, dict):
            reason_codes.append("matrix_wrapper_entry_invalid")
            continue
        wrapper_loc, impl_loc = _check_wrapper(
            root,
            entry,
            expected_dispatch_target,
            reason_codes,
        )
        total_wrapper_shell_loc += wrapper_loc
        total_impl_shell_loc += impl_loc

    if total_wrapper_shell_loc > max_wrapper_shell_loc:
        reason_codes.append("service_api_tranche2_wrapper_shell_loc_budget_exceeded")
    if total_impl_shell_loc > max_impl_shell_loc:
        reason_codes.append("service_api_tranche2_impl_shell_loc_budget_exceeded")

    if reason_codes:
        reason_codes_value = ",".join(reason_codes)
        report = {
            "schema_version": REPORT_SCHEMA,
            "status": "fail",
            "service_api_tranche2_wrapper_family_status": "rejected",
            "wrapper_count": len(wrappers),
            "wrapper_shell_loc": total_wrapper_shell_loc,
            "max_wrapper_shell_loc": max_wrapper_shell_loc,
            "impl_shell_loc": total_impl_shell_loc,
            "max_impl_shell_loc": max_impl_shell_loc,
            "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
            "reason_codes_csv": REASON_CODES_CSV,
            "reason_codes": reason_codes,
            "reason_codes_value": reason_codes_value,
        }
        _write_report(args.output_json, report)
        print("status=fail")
        print("service_api_tranche2_wrapper_family_status=rejected")
        print(f"wrapper_count={len(wrappers)}")
        print(f"wrapper_shell_loc={total_wrapper_shell_loc}")
        print(f"max_wrapper_shell_loc={max_wrapper_shell_loc}")
        print(f"impl_shell_loc={total_impl_shell_loc}")
        print(f"max_impl_shell_loc={max_impl_shell_loc}")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print(f"reason_codes_csv={REASON_CODES_CSV}")
        print(f"reason_codes={reason_codes_value}")
        raise SystemExit(1)

    report = {
        "schema_version": REPORT_SCHEMA,
        "status": "pass",
        "service_api_tranche2_wrapper_family_status": "verified",
        "wrapper_count": len(wrappers),
        "wrapper_shell_loc": total_wrapper_shell_loc,
        "max_wrapper_shell_loc": max_wrapper_shell_loc,
        "impl_shell_loc": total_impl_shell_loc,
        "max_impl_shell_loc": max_impl_shell_loc,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes": [],
        "reason_codes_value": "none",
    }
    _write_report(args.output_json, report)
    print("status=pass")
    print("service_api_tranche2_wrapper_family_status=verified")
    print(f"wrapper_count={len(wrappers)}")
    print(f"wrapper_shell_loc={total_wrapper_shell_loc}")
    print(f"max_wrapper_shell_loc={max_wrapper_shell_loc}")
    print(f"impl_shell_loc={total_impl_shell_loc}")
    print(f"max_impl_shell_loc={max_impl_shell_loc}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print("reason_codes=none")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate service API tranche-2 wrapper retirement parity contract.",
    )
    parser.add_argument("--root-dir", required=True, help="Repository root.")
    parser.add_argument("--matrix-file", required=True, help="Wrapper family matrix JSON.")
    parser.add_argument(
        "--output-json",
        help="Optional path to write deterministic parity report JSON.",
    )
    args = parser.parse_args()
    return _run(args)


if __name__ == "__main__":
    raise SystemExit(main())
