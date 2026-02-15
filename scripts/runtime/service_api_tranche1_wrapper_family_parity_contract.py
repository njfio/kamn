#!/usr/bin/env python3
"""Check service API tranche-1 wrapper family parity and migration invariants."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
from typing import Any

MATRIX_SCHEMA = "kamn.runtime.service-api-tranche1-wrapper-family-matrix.v1"


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


def _is_executable(path: Path) -> bool:
    return path.exists() and path.is_file() and os.access(path, os.X_OK)


def _ensure_string(payload: dict[str, Any], key: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise SystemExit(f"matrix_field_invalid:{key}")
    return value


def _check_wrapper(root: Path, wrapper_cfg: dict[str, Any], reason_codes: list[str]) -> int:
    wrapper = _ensure_string(wrapper_cfg, "wrapper")
    validation_script = _ensure_string(wrapper_cfg, "validation_script")
    policy_checker = _ensure_string(wrapper_cfg, "policy_checker")
    contract_status_key = _ensure_string(wrapper_cfg, "contract_status_key")
    policy_status_key = _ensure_string(wrapper_cfg, "policy_status_key")
    tamper_reason_code = _ensure_string(wrapper_cfg, "tamper_reason_code")

    wrapper_path = root / wrapper
    if not wrapper_path.exists():
        reason_codes.append(f"wrapper_missing:{wrapper}")
        return 0
    if not _is_executable(wrapper_path):
        reason_codes.append(f"wrapper_not_executable:{wrapper}")
        return 0

    text = _read_text(wrapper_path)
    if 'source "$ROOT_DIR/scripts/runtime/service_api_contract_lane_runner.sh"' not in text:
        reason_codes.append(f"wrapper_runner_source_marker_missing:{wrapper}")
    if 'service_api_contract_lane_run "$@"' not in text:
        reason_codes.append(f"wrapper_runner_entry_marker_missing:{wrapper}")
    if f'VALIDATION_SCRIPT="$ROOT_DIR/{validation_script}"' not in text:
        reason_codes.append(f"wrapper_validation_script_marker_missing:{wrapper}")
    if f'POLICY_CHECKER="$ROOT_DIR/{policy_checker}"' not in text:
        reason_codes.append(f"wrapper_policy_checker_marker_missing:{wrapper}")
    if f'CONTRACT_STATUS_KEY="{contract_status_key}"' not in text:
        reason_codes.append(f"wrapper_contract_status_marker_missing:{wrapper}")
    if f'POLICY_STATUS_KEY="{policy_status_key}"' not in text:
        reason_codes.append(f"wrapper_policy_status_marker_missing:{wrapper}")
    if f'TAMPER_REASON_CODE="{tamper_reason_code}"' not in text:
        reason_codes.append(f"wrapper_tamper_reason_marker_missing:{wrapper}")

    return sum(1 for _ in wrapper_path.read_text(encoding="utf-8").splitlines())


def _run(args: argparse.Namespace) -> int:
    root = Path(args.root_dir).resolve()
    matrix_file = Path(args.matrix_file).resolve()
    payload = _load_json(matrix_file)

    if payload.get("schema_version") != MATRIX_SCHEMA:
        raise SystemExit("matrix_schema_mismatch")

    wrappers = payload.get("wrappers")
    if not isinstance(wrappers, list) or not wrappers:
        raise SystemExit("matrix_wrappers_missing")

    max_shell_loc = payload.get("max_shell_loc")
    if not isinstance(max_shell_loc, int) or max_shell_loc < 1:
        raise SystemExit("matrix_max_shell_loc_invalid")

    reason_codes: list[str] = []
    total_shell_loc = 0
    for entry in wrappers:
        if not isinstance(entry, dict):
            reason_codes.append("matrix_wrapper_entry_invalid")
            continue
        total_shell_loc += _check_wrapper(root, entry, reason_codes)

    if total_shell_loc > max_shell_loc:
        reason_codes.append("service_api_tranche1_shell_loc_budget_exceeded")

    if reason_codes:
        reason_codes_csv = ",".join(reason_codes)
        print("status=fail")
        print("service_api_tranche1_wrapper_family_status=rejected")
        print(f"wrapper_count={len(wrappers)}")
        print(f"total_shell_loc={total_shell_loc}")
        print(f"max_shell_loc={max_shell_loc}")
        print(f"reason_codes={reason_codes_csv}")
        raise SystemExit(1)

    print("status=pass")
    print("service_api_tranche1_wrapper_family_status=verified")
    print(f"wrapper_count={len(wrappers)}")
    print(f"total_shell_loc={total_shell_loc}")
    print(f"max_shell_loc={max_shell_loc}")
    print("reason_codes=none")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(
        description="Validate service API tranche-1 wrapper family parity contract.",
    )
    parser.add_argument("--root-dir", required=True, help="Repository root.")
    parser.add_argument("--matrix-file", required=True, help="Wrapper family matrix JSON.")
    args = parser.parse_args()
    return _run(args)


if __name__ == "__main__":
    raise SystemExit(main())
