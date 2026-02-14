#!/usr/bin/env python3
"""Production go/no-go gate lane contract runner."""

from __future__ import annotations

import argparse
import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import time

SCRIPT_DIR = Path(__file__).resolve().parent
ROOT_DIR = SCRIPT_DIR.parent.parent
sys.path.insert(0, str(SCRIPT_DIR.parent))

from framework.contract_framework import ContractError, fail, require_non_negative_int, write_json  # noqa: E402

GATE_DECISION_FAULT_REASON = "gate_decision_fault_injection_triggered"
RELEASE_MANIFEST_SCHEMA = "kamn.runtime.release-evidence-manifest.v1"
DEFAULT_RELEASE_MANIFEST_PATH = ROOT_DIR / "scripts/runtime/release_evidence_manifest.json"
REQUIRED_ARTIFACT_IDS = (
    "go_no_go_evidence",
    "rollback_readiness",
    "dr_readiness",
)
ARTIFACT_LANE_REGISTRY: dict[str, dict[str, object]] = {
    "go_no_go_evidence": {
        "expected_lane": "deploy.run_gonogo_evidence_deep_lane",
        "command": ["bash", str(ROOT_DIR / "scripts/deploy/run_gonogo_evidence_deep_lane.sh")],
        "success_marker": "go/no-go evidence deep lane tests passed.",
        "failure_label": "go/no-go evidence lane failed unexpectedly",
    },
    "rollback_readiness": {
        "expected_lane": "deploy.run_deployment_slo_rollback_contract_lane",
        "command": [
            "bash",
            str(ROOT_DIR / "scripts/deploy/run_deployment_slo_rollback_contract_lane.sh"),
        ],
        "success_marker": "final_decision=GO",
        "failure_label": "rollback readiness lane failed unexpectedly",
    },
    "dr_readiness": {
        "expected_lane": "deploy.run_dr_evidence_contract_lane",
        "command": ["bash", str(ROOT_DIR / "scripts/deploy/run_dr_evidence_contract_lane.sh")],
        "success_marker": "dr evidence contract lane tests passed.",
        "failure_label": "dr readiness lane failed unexpectedly",
    },
}


def _ensure_executable(path: Path, label: str) -> None:
    if not path.is_file() or not os.access(path, os.X_OK):
        fail(f"expected {label} to be executable")


def _run_command(command: list[str]) -> subprocess.CompletedProcess[str]:
    return subprocess.run(
        command,
        cwd=ROOT_DIR,
        capture_output=True,
        text=True,
        check=False,
    )


def _resolve_manifest_path(raw: str) -> Path:
    value = raw.strip()
    if not value:
        return DEFAULT_RELEASE_MANIFEST_PATH
    path = Path(value)
    if path.is_absolute():
        return path
    return (ROOT_DIR / path).resolve()


def _load_release_manifest(path: Path) -> dict[str, object]:
    if not path.is_file():
        fail("release_manifest_file_missing")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError:
        fail("release_manifest_json_invalid")
    if not isinstance(payload, dict):
        fail("release_manifest_root_invalid")
    return payload


def _require_non_empty_string(payload: dict[str, object], key: str, reason_code: str) -> str:
    value = payload.get(key)
    if not isinstance(value, str) or not value.strip():
        fail(reason_code)
    return value.strip()


def _validate_release_manifest(payload: dict[str, object]) -> list[dict[str, str]]:
    schema_version = payload.get("schema_version")
    if schema_version != RELEASE_MANIFEST_SCHEMA:
        fail("release_manifest_schema_version_invalid")

    required_artifacts = payload.get("required_artifacts")
    if not isinstance(required_artifacts, list):
        fail("release_manifest_required_artifacts_missing")

    seen_artifact_ids: set[str] = set()
    validated_artifacts: list[dict[str, str]] = []
    for entry in required_artifacts:
        if not isinstance(entry, dict):
            fail("release_manifest_required_artifact_entry_invalid")
        artifact_id = _require_non_empty_string(
            entry,
            "artifact_id",
            "release_manifest_required_artifact_id_invalid",
        )
        if artifact_id in seen_artifact_ids:
            fail(f"release_manifest_duplicate_artifact_id:{artifact_id}")
        if artifact_id not in ARTIFACT_LANE_REGISTRY:
            fail(f"release_manifest_unknown_artifact_id:{artifact_id}")

        expected_lane = _require_non_empty_string(
            entry,
            "expected_lane",
            f"release_manifest_expected_lane_missing:{artifact_id}",
        )
        expected_success_marker = _require_non_empty_string(
            entry,
            "expected_success_marker",
            f"release_manifest_expected_success_marker_missing:{artifact_id}",
        )
        registry_entry = ARTIFACT_LANE_REGISTRY[artifact_id]
        if expected_lane != registry_entry["expected_lane"]:
            fail(f"release_manifest_lane_mismatch:{artifact_id}")
        if expected_success_marker != registry_entry["success_marker"]:
            fail(f"release_manifest_success_marker_mismatch:{artifact_id}")

        seen_artifact_ids.add(artifact_id)
        validated_artifacts.append(
            {
                "artifact_id": artifact_id,
                "expected_lane": expected_lane,
                "expected_success_marker": expected_success_marker,
            }
        )

    for required_artifact in REQUIRED_ARTIFACT_IDS:
        if required_artifact not in seen_artifact_ids:
            fail(f"release_manifest_missing_required_artifact:{required_artifact}")

    return validated_artifacts


def run_go_no_go_gate_lane(args: argparse.Namespace) -> int:
    fault_profile = args.fault_profile.strip()
    if fault_profile not in {"none", "gate_decision"}:
        fail("--fault-profile must be one of: none, gate_decision")

    max_seconds = require_non_negative_int("KAMN_GONOGO_GATE_MAX_SECONDS", args.max_seconds)
    start_epoch = int(time.time())
    manifest_path = _resolve_manifest_path(args.manifest_file)
    manifest_payload = _load_release_manifest(manifest_path)
    required_artifacts = _validate_release_manifest(manifest_payload)

    gonogo_generator = ROOT_DIR / "scripts/deploy/generate_gonogo_evidence_bundle.sh"
    gonogo_checker = ROOT_DIR / "scripts/deploy/check_gonogo_evidence_policy.sh"

    _ensure_executable(gonogo_generator, "go/no-go evidence bundle generator")
    _ensure_executable(gonogo_checker, "go/no-go evidence policy checker")

    reason_codes: list[str] = []
    artifact_inventory: list[dict[str, str]] = []

    for artifact in required_artifacts:
        artifact_id = artifact["artifact_id"]
        registry_entry = ARTIFACT_LANE_REGISTRY[artifact_id]
        lane_command = registry_entry["command"]
        if not isinstance(lane_command, list) or len(lane_command) < 2:
            fail(f"release_manifest_command_invalid:{artifact_id}")
        lane_path = Path(str(lane_command[1]))
        _ensure_executable(lane_path, f"{artifact_id} lane command")
        run_result = _run_command([str(part) for part in lane_command])
        if run_result.returncode != 0:
            detail = (
                run_result.stderr
                or run_result.stdout
                or f"{registry_entry['failure_label']}: unknown failure"
            ).strip()
            fail(f"release_manifest_artifact_execution_failed:{artifact_id}:{detail}")
        expected_marker = artifact["expected_success_marker"]
        if expected_marker not in run_result.stdout:
            fail(f"release_manifest_required_marker_missing:{artifact_id}")
        artifact_inventory.append(
            {
                "artifact_id": artifact_id,
                "expected_lane": artifact["expected_lane"],
                "expected_success_marker": expected_marker,
                "status": "verified",
            }
        )

    if fault_profile == "gate_decision":
        with tempfile.TemporaryDirectory() as temp_dir:
            temp_path = Path(temp_dir)
            no_go_bundle = temp_path / "gonogo-no-go-bundle.json"
            drift_bundle = temp_path / "gonogo-drift-bundle.json"

            generate_run = _run_command(
                [
                    "bash",
                    str(gonogo_generator),
                    "--output-file",
                    str(no_go_bundle),
                    "--release-candidate",
                    "v1.0.0-fault",
                    "--schema-target-version",
                    "1.0.0",
                    "--runtime-image-digest",
                    "sha256:fault",
                    "--ci-fast-gate",
                    "PASS",
                    "--ci-deep-lane",
                    "FAIL",
                    "--rollback-precheck",
                    "PASS",
                    "--rollback-trigger-status",
                    "CLEAR",
                    "--required-approvals",
                    "2",
                    "--received-approvals",
                    "1",
                ]
            )
            if generate_run.returncode != 0:
                detail = (generate_run.stderr or generate_run.stdout or "generator failed").strip()
                fail(f"go/no-go generator failed unexpectedly: {detail}")

            payload = json.loads(no_go_bundle.read_text(encoding="utf-8"))
            payload["final_decision"] = "GO"
            drift_bundle.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

            drift_run = _run_command(
                [
                    "bash",
                    str(gonogo_checker),
                    "--bundle-file",
                    str(drift_bundle),
                ]
            )
            if drift_run.returncode == 0:
                fail("gate_decision fault profile expected policy checker fail-closed behavior")
            reason_codes.append(GATE_DECISION_FAULT_REASON)

    elapsed_seconds = int(time.time()) - start_epoch
    if elapsed_seconds > max_seconds:
        reason_codes.append("runtime_budget_exceeded")

    reason_codes = sorted(set(reason_codes))
    status = "pass"
    final_decision = "GO"
    if reason_codes:
        status = "fail"
        final_decision = "NO-GO"

    report_payload = {
        "schema_version": "kamn.runtime.go-no-go-gate-report.v1",
        "status": status,
        "final_decision": final_decision,
        "fault_profile": fault_profile,
        "go_no_go_evidence_status": "verified",
        "rollback_readiness_status": "verified",
        "dr_readiness_status": "verified",
        "manifest_schema_version": manifest_payload["schema_version"],
        "manifest_registry_status": "verified",
        "required_artifact_ids": [artifact["artifact_id"] for artifact in required_artifacts],
        "artifact_inventory": artifact_inventory,
        "reason_codes": reason_codes,
        "elapsed_seconds": elapsed_seconds,
        "max_seconds": max_seconds,
    }
    if args.output_json:
        write_json(Path(args.output_json), report_payload)

    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"fault_profile={fault_profile}")
    print("go_no_go_evidence_status=verified")
    print("rollback_readiness_status=verified")
    print("dr_readiness_status=verified")
    print(f"manifest_schema_version={manifest_payload['schema_version']}")
    print("manifest_registry_status=verified")
    print(f"required_artifact_count={len(required_artifacts)}")
    print(f"reason_codes={reason_codes_csv}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")

    if status != "pass":
        fail(f"go/no-go gate lane failed closed: {reason_codes_csv}")

    print("go/no-go gate lane tests passed.")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description="Run production go/no-go gate lane.")
    parser.add_argument(
        "--fault-profile",
        default=os.environ.get("KAMN_GONOGO_GATE_FAULT_PROFILE", "none"),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument(
        "--max-seconds",
        default=os.environ.get("KAMN_GONOGO_GATE_MAX_SECONDS", "180"),
    )
    parser.add_argument(
        "--manifest-file",
        default=os.environ.get(
            "KAMN_GONOGO_GATE_MANIFEST_FILE",
            str(DEFAULT_RELEASE_MANIFEST_PATH),
        ),
    )
    parser.set_defaults(handler=run_go_no_go_gate_lane)
    return parser


def main(argv: list[str]) -> int:
    parser = build_parser()
    args = parser.parse_args(argv)
    return args.handler(args)


if __name__ == "__main__":
    try:
        raise SystemExit(main(sys.argv[1:]))
    except ContractError as error:
        print(error, file=sys.stderr)
        raise SystemExit(1)
