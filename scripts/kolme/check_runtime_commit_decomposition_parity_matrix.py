#!/usr/bin/env python3
"""Validate runtime commit decomposition parity matrix artifacts."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

MATRIX_SCHEMA_VERSION = "kamn.kolme.runtime-commit-decomposition-parity-matrix.v1"
POLICY_SCHEMA_VERSION = "kamn.kolme.runtime-commit-decomposition-parity-policy.v1"
REQUIRED_SCENARIO_IDS = {
    "submit_http_round_trip",
    "submit_fork_broadcast_round_trip",
    "finality_notification_resolution",
    "finality_block_fallback_resolution",
}


def repo_root() -> Path:
    return Path(__file__).resolve().parents[2]


def load_json_object(path: Path, *, label: str) -> dict[str, Any]:
    if not path.is_file():
        raise ValueError(f"{label} not found: {path}")
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError(f"{label} must be a JSON object: {path}")
    return payload


def require_string(payload: dict[str, Any], key: str, *, label: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ValueError(f"{label} {key} must be a non-empty string")
    return value.strip()


def require_list(payload: dict[str, Any], key: str, *, label: str) -> list[Any]:
    value = payload.get(key)
    if not isinstance(value, list) or not value:
        raise ValueError(f"{label} {key} must be a non-empty array")
    return value


def write_json(path: Path, payload: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def check(args: argparse.Namespace) -> int:
    root = repo_root()
    matrix_file = Path(args.matrix_file).resolve()
    output_json = Path(args.output_json).resolve()

    payload = load_json_object(matrix_file, label="runtime commit decomposition parity matrix")
    if payload.get("schema_version") != MATRIX_SCHEMA_VERSION:
        raise ValueError(f"schema_version must be {MATRIX_SCHEMA_VERSION}")

    legacy_surface_file = require_string(payload, "legacy_surface_file", label="matrix")
    extracted_surface_root = require_string(payload, "extracted_surface_root", label="matrix")
    scenarios = require_list(payload, "scenarios", label="matrix")

    violations: list[str] = []
    scenario_ids_seen: set[str] = set()

    legacy_surface_path = root / legacy_surface_file
    if not legacy_surface_path.is_file():
        violations.append(f"legacy surface file missing: {legacy_surface_file}")

    extracted_surface_path = root / extracted_surface_root
    if not extracted_surface_path.is_dir():
        violations.append(f"extracted surface root missing: {extracted_surface_root}")

    for idx, scenario_raw in enumerate(scenarios):
        if not isinstance(scenario_raw, dict):
            violations.append(f"scenario[{idx}] must be an object")
            continue

        scenario_id = require_string(scenario_raw, "scenario_id", label=f"scenario[{idx}]")
        if scenario_id in scenario_ids_seen:
            violations.append(f"duplicate scenario_id: {scenario_id}")
        scenario_ids_seen.add(scenario_id)

        require_string(scenario_raw, "legacy_contract", label=f"scenario[{idx}]")
        parity_status = require_string(scenario_raw, "parity_status", label=f"scenario[{idx}]")
        if parity_status != "preserved":
            violations.append(
                f"scenario[{scenario_id}] parity_status must be preserved (found {parity_status})"
            )

        extracted_modules = require_list(
            scenario_raw, "extracted_module_files", label=f"scenario[{idx}]"
        )
        for module_file in extracted_modules:
            if not isinstance(module_file, str) or not module_file.strip():
                violations.append(
                    f"scenario[{scenario_id}] extracted_module_files entries must be non-empty strings"
                )
                continue
            module_path = root / module_file
            if not module_path.is_file():
                violations.append(
                    f"scenario[{scenario_id}] missing extracted module file: {module_file}"
                )

        evidence_tests = require_list(scenario_raw, "evidence_tests", label=f"scenario[{idx}]")
        for evidence_test in evidence_tests:
            if not isinstance(evidence_test, str) or not evidence_test.strip():
                violations.append(
                    f"scenario[{scenario_id}] evidence_tests entries must be non-empty strings"
                )
                continue
            evidence_path = root / evidence_test
            if not evidence_path.exists():
                violations.append(
                    f"scenario[{scenario_id}] missing evidence test path: {evidence_test}"
                )

    missing_required = sorted(REQUIRED_SCENARIO_IDS - scenario_ids_seen)
    if missing_required:
        violations.append(f"missing required scenarios: {', '.join(missing_required)}")

    policy = {
        "schema_version": POLICY_SCHEMA_VERSION,
        "status": "pass" if not violations else "fail",
        "final_decision": "GO" if not violations else "HOLD",
        "reason_key": "runtime_commit_decomposition_parity_preserved"
        if not violations
        else "runtime_commit_decomposition_parity_drift_detected",
        "matrix_file": str(matrix_file),
        "legacy_surface_file": legacy_surface_file,
        "extracted_surface_root": extracted_surface_root,
        "scenario_count": len(scenario_ids_seen),
        "required_scenario_count": len(REQUIRED_SCENARIO_IDS),
        "violation_count": len(violations),
        "violations": sorted(violations),
    }
    write_json(output_json, policy)

    out = sys.stdout if not violations else sys.stderr
    print(f"status={policy['status']}", file=out)
    print(f"final_decision={policy['final_decision']}", file=out)
    print(f"reason_key={policy['reason_key']}", file=out)
    print(f"scenario_count={policy['scenario_count']}", file=out)
    print(f"violation_count={policy['violation_count']}", file=out)
    for violation in policy["violations"]:
        print(violation, file=out)

    return 0 if not violations else 1


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Validate runtime commit decomposition parity matrix artifacts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    check_parser = subparsers.add_parser("check")
    check_parser.add_argument("--matrix-file", required=True)
    check_parser.add_argument("--output-json", required=True)

    return parser


def main() -> int:
    parser = build_parser()
    args = parser.parse_args()
    try:
        if args.command == "check":
            return check(args)
    except ValueError as error:
        print("status=fail", file=sys.stderr)
        print("final_decision=HOLD", file=sys.stderr)
        print(f"runtime commit decomposition parity policy failed: {error}", file=sys.stderr)
        return 1
    raise AssertionError(f"unsupported command: {args.command}")


if __name__ == "__main__":
    raise SystemExit(main())
