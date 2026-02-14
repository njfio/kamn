#!/usr/bin/env python3
"""Generate and validate deterministic on-chain lifecycle evidence bundles."""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path
from typing import Any

BUNDLE_SCHEMA_VERSION = "kamn.kolme.onchain-lifecycle-evidence-bundle.v1"
POLICY_SCHEMA_VERSION = "kamn.kolme.onchain-lifecycle-evidence-policy-report.v1"

GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"

ARTIFACT_SPECS: tuple[dict[str, str], ...] = (
    {
        "id": "did_lifecycle",
        "schema_version": "kamn.kolme.did-lifecycle-chain.live-validation.v1",
        "finality_marker": "did_lifecycle_contract_status",
        "recovery_marker": "fail_closed_status",
    },
    {
        "id": "message_proof",
        "schema_version": "kamn.kolme.message-proof-anchoring.live-validation.v1",
        "finality_marker": "message_anchor_contract_status",
        "recovery_marker": "fail_closed_status",
    },
    {
        "id": "runtime_commit",
        "schema_version": "kamn.kolme.continuous-runtime-commit.live-validation.v1",
        "finality_marker": "continuous_runtime_contract_status",
        "recovery_marker": "fail_closed_status",
    },
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Generate/check deterministic on-chain lifecycle evidence policy artifacts."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--mode", required=True, choices=["dry-run", "run"])
    generate.add_argument("--did-report-file", required=True)
    generate.add_argument("--message-report-file", required=True)
    generate.add_argument("--runtime-report-file", required=True)
    generate.add_argument("--output-json", required=True)
    generate.add_argument("--max-seconds", required=True)
    generate.add_argument("--elapsed-seconds", required=True)
    generate.add_argument(
        "--budget-status",
        required=True,
        choices=["not_run", "within_budget", "exceeded_budget"],
    )
    generate.add_argument("--reason-code", required=True)

    check = subparsers.add_parser("check")
    check.add_argument("--report-file", required=True)
    check.add_argument("--expected-final-decision", required=True, choices=[GO_DECISION, NO_GO_DECISION])
    check.add_argument("--ci-fast-gate", required=True, choices=["PASS", "FAIL"])
    check.add_argument("--require-reason-code", action="append", default=[])
    check.add_argument("--output-json", default="")
    return parser.parse_args()


def _artifact_spec_by_id(artifact_id: str) -> dict[str, str] | None:
    for spec in ARTIFACT_SPECS:
        if spec["id"] == artifact_id:
            return spec
    return None


def _build_artifact_entry(spec: dict[str, str], report_path: Path) -> tuple[dict[str, Any], list[str]]:
    artifact_id = spec["id"]
    entry: dict[str, Any] = {
        "id": artifact_id,
        "report_file": str(report_path.resolve()),
        "schema_version": "",
        "sha256": "",
        "status": "",
        "final_decision": "",
        "finality_marker": spec["finality_marker"],
        "finality_marker_status": "missing",
        "recovery_marker": spec["recovery_marker"],
        "recovery_marker_status": "missing",
        "fail_closed_reason_code": "",
    }
    reason_codes: list[str] = []

    if not report_path.is_file():
        reason_codes.append(f"linked_artifact_missing:{artifact_id}")
        reason_codes.append(f"finality_lineage_missing:{artifact_id}")
        reason_codes.append(f"recovery_lineage_missing:{artifact_id}")
        return entry, reason_codes

    raw_payload = report_path.read_bytes()
    entry["sha256"] = hashlib.sha256(raw_payload).hexdigest()
    try:
        payload = json.loads(raw_payload.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError):
        reason_codes.append(f"linked_artifact_json_invalid:{artifact_id}")
        reason_codes.append(f"finality_lineage_missing:{artifact_id}")
        reason_codes.append(f"recovery_lineage_missing:{artifact_id}")
        return entry, reason_codes

    entry["schema_version"] = str(payload.get("schema_version", ""))
    entry["status"] = str(payload.get("status", ""))
    entry["final_decision"] = str(payload.get("final_decision", ""))
    fail_closed_reason_code = payload.get("fail_closed_reason_code")
    if isinstance(fail_closed_reason_code, str):
        entry["fail_closed_reason_code"] = fail_closed_reason_code

    if entry["schema_version"] != spec["schema_version"]:
        reason_codes.append(f"linked_artifact_schema_mismatch:{artifact_id}")
    if entry["status"] != "pass":
        reason_codes.append(f"linked_artifact_status_mismatch:{artifact_id}")
    if entry["final_decision"] != GO_DECISION:
        reason_codes.append(f"linked_artifact_final_decision_mismatch:{artifact_id}")

    finality_value = payload.get(spec["finality_marker"])
    if finality_value == "verified":
        entry["finality_marker_status"] = "verified"
    else:
        reason_codes.append(f"finality_lineage_missing:{artifact_id}")

    recovery_value = payload.get(spec["recovery_marker"])
    if recovery_value == "verified":
        entry["recovery_marker_status"] = "verified"
    else:
        reason_codes.append(f"recovery_lineage_missing:{artifact_id}")

    return entry, reason_codes


def _build_bundle_payload(
    mode: str,
    did_report: Path,
    message_report: Path,
    runtime_report: Path,
    max_seconds: int,
    elapsed_seconds: int,
    budget_status: str,
    reason_code: str,
) -> dict[str, Any]:
    artifact_paths = {
        "did_lifecycle": did_report,
        "message_proof": message_report,
        "runtime_commit": runtime_report,
    }

    linked_artifacts: list[dict[str, Any]] = []
    reason_codes: list[str] = []
    for spec in ARTIFACT_SPECS:
        artifact_id = spec["id"]
        entry, artifact_reasons = _build_artifact_entry(spec, artifact_paths[artifact_id])
        linked_artifacts.append(entry)
        reason_codes.extend(artifact_reasons)

    finality_lineage_status = (
        "verified"
        if not any(code.startswith("finality_lineage_missing:") for code in reason_codes)
        else "missing"
    )
    recovery_lineage_status = (
        "verified"
        if not any(code.startswith("recovery_lineage_missing:") for code in reason_codes)
        else "missing"
    )

    status = "ok" if not reason_codes else "fail"
    final_decision = GO_DECISION if status == "ok" else NO_GO_DECISION

    normalized_artifact_paths = [str(path.resolve()) for path in artifact_paths.values()]

    payload: dict[str, Any] = {
        "schema_version": BUNDLE_SCHEMA_VERSION,
        "mode": mode,
        "status": status,
        "final_decision": final_decision,
        "reason_code": reason_code,
        "reason_codes": reason_codes,
        "local_only_enforced": True,
        "ci_fast_gate_eligible": False,
        "max_seconds": max_seconds,
        "elapsed_seconds": elapsed_seconds,
        "budget_status": budget_status,
        "finality_lineage_status": finality_lineage_status,
        "recovery_lineage_status": recovery_lineage_status,
        "linked_artifacts": linked_artifacts,
        "artifact_paths": normalized_artifact_paths,
        "contracts": {
            "ci_fast_gate_scope": "local-only",
            "bundle_contract": "onchain_lifecycle_evidence_bundle_v1",
            "finality_lineage_required": True,
            "recovery_lineage_required": True,
            "linked_artifact_integrity_contract": "sha256",
        },
    }
    return payload


def _build_expected_from_bundle(bundle: dict[str, Any]) -> tuple[dict[str, Any], list[str]]:
    reason_codes: list[str] = []
    mode = str(bundle.get("mode", ""))
    if mode not in {"dry-run", "run"}:
        reason_codes.append("mode_invalid")

    max_seconds = bundle.get("max_seconds")
    if not isinstance(max_seconds, int) or max_seconds <= 0:
        reason_codes.append("max_seconds_invalid")
        max_seconds = 1

    elapsed_seconds = bundle.get("elapsed_seconds")
    if not isinstance(elapsed_seconds, int) or elapsed_seconds < 0:
        reason_codes.append("elapsed_seconds_invalid")
        elapsed_seconds = 0

    budget_status = bundle.get("budget_status")
    if budget_status not in {"not_run", "within_budget", "exceeded_budget"}:
        reason_codes.append("budget_status_invalid")
        budget_status = "not_run"

    linked_artifacts = bundle.get("linked_artifacts")
    if not isinstance(linked_artifacts, list):
        linked_artifacts = []
        reason_codes.append("linked_artifacts_invalid")

    reported_paths: dict[str, Path] = {}
    for entry in linked_artifacts:
        if not isinstance(entry, dict):
            reason_codes.append("linked_artifact_entry_invalid")
            continue
        artifact_id = entry.get("id")
        if not isinstance(artifact_id, str):
            reason_codes.append("linked_artifact_id_invalid")
            continue
        report_file = entry.get("report_file")
        if not isinstance(report_file, str) or report_file.strip() == "":
            reason_codes.append(f"linked_artifact_report_file_missing:{artifact_id}")
            continue
        reported_paths[artifact_id] = Path(report_file).resolve()

    expected_path_map: dict[str, Path] = {}
    for spec in ARTIFACT_SPECS:
        artifact_id = spec["id"]
        path = reported_paths.get(artifact_id)
        if path is None:
            reason_codes.append(f"linked_artifact_missing:{artifact_id}")
            path = Path(f"/tmp/missing_{artifact_id}.json")
        expected_path_map[artifact_id] = path

    expected = _build_bundle_payload(
        mode=mode if mode in {"dry-run", "run"} else "dry-run",
        did_report=expected_path_map["did_lifecycle"],
        message_report=expected_path_map["message_proof"],
        runtime_report=expected_path_map["runtime_commit"],
        max_seconds=max_seconds,
        elapsed_seconds=elapsed_seconds,
        budget_status=str(budget_status),
        reason_code=str(bundle.get("reason_code", "")),
    )
    reason_codes.extend(expected["reason_codes"])
    return expected, reason_codes


def _canonical_linked_artifacts(bundle: dict[str, Any]) -> list[dict[str, Any]]:
    linked_artifacts = bundle.get("linked_artifacts")
    if not isinstance(linked_artifacts, list):
        return []

    allowed_keys = (
        "id",
        "report_file",
        "schema_version",
        "sha256",
        "status",
        "final_decision",
        "finality_marker",
        "finality_marker_status",
        "recovery_marker",
        "recovery_marker_status",
        "fail_closed_reason_code",
    )
    canonical: list[dict[str, Any]] = []
    for entry in linked_artifacts:
        if not isinstance(entry, dict):
            continue
        canonical.append({key: entry.get(key) for key in allowed_keys})
    canonical.sort(key=lambda item: str(item.get("id", "")))
    return canonical


def generate_bundle(args: argparse.Namespace) -> int:
    max_seconds = int(args.max_seconds)
    elapsed_seconds = int(args.elapsed_seconds)
    if max_seconds <= 0:
        raise SystemExit("max-seconds must be greater than zero")
    if elapsed_seconds < 0:
        raise SystemExit("elapsed-seconds must be non-negative")

    payload = _build_bundle_payload(
        mode=args.mode,
        did_report=Path(args.did_report_file).resolve(),
        message_report=Path(args.message_report_file).resolve(),
        runtime_report=Path(args.runtime_report_file).resolve(),
        max_seconds=max_seconds,
        elapsed_seconds=elapsed_seconds,
        budget_status=args.budget_status,
        reason_code=args.reason_code,
    )

    output_path = Path(args.output_json).resolve()
    output_path.parent.mkdir(parents=True, exist_ok=True)
    output_path.write_text(json.dumps(payload, sort_keys=True, indent=2) + "\n", encoding="utf-8")

    print(f"status={payload['status']}")
    print(f"final_decision={payload['final_decision']}")
    print(f"reason_code={payload['reason_code']}")
    print(f"finality_lineage_status={payload['finality_lineage_status']}")
    print(f"recovery_lineage_status={payload['recovery_lineage_status']}")
    print(f"bundle_file={output_path}")
    return 0 if payload["final_decision"] == GO_DECISION else 1


def check_bundle(args: argparse.Namespace) -> int:
    report_path = Path(args.report_file).resolve()
    payload = json.loads(report_path.read_text(encoding="utf-8"))

    reason_codes: list[str] = []
    if payload.get("schema_version") != BUNDLE_SCHEMA_VERSION:
        reason_codes.append("schema_version_mismatch")
    if payload.get("local_only_enforced") is not True:
        reason_codes.append("local_only_enforced_missing")

    ci_fast_gate_eligible = payload.get("ci_fast_gate_eligible")
    if ci_fast_gate_eligible is not False:
        reason_codes.append("ci_fast_gate_eligibility_violation")
    if args.ci_fast_gate != "PASS":
        reason_codes.append("ci_fast_gate_failed")

    expected_payload, expected_reason_codes = _build_expected_from_bundle(payload)
    reason_codes.extend(code for code in expected_reason_codes if code not in reason_codes)

    canonical_actual_artifacts = _canonical_linked_artifacts(payload)
    canonical_expected_artifacts = _canonical_linked_artifacts(expected_payload)
    if canonical_actual_artifacts != canonical_expected_artifacts:
        reason_codes.append("aggregate_bundle_lineage_mismatch")

    for field_name in (
        "finality_lineage_status",
        "recovery_lineage_status",
        "status",
        "final_decision",
    ):
        if payload.get(field_name) != expected_payload.get(field_name):
            reason_codes.append(f"{field_name}_mismatch")

    actual_reason_codes = payload.get("reason_codes")
    if not isinstance(actual_reason_codes, list):
        reason_codes.append("reason_codes_invalid")
    elif actual_reason_codes != expected_payload.get("reason_codes"):
        reason_codes.append("aggregate_bundle_reason_code_drift")

    observed_reason_code = payload.get("reason_code")
    if not isinstance(observed_reason_code, str) or observed_reason_code.strip() == "":
        reason_codes.append("reason_code_missing")
    for required_reason in args.require_reason_code:
        if observed_reason_code != required_reason:
            reason_codes.append(f"required_reason_code_missing:{required_reason}")

    observed_final_decision = payload.get("final_decision")
    if observed_final_decision != args.expected_final_decision:
        reason_codes.append("observed_final_decision_mismatch")

    final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION
    output = {
        "schema_version": POLICY_SCHEMA_VERSION,
        "report_file": str(report_path),
        "expected_final_decision": args.expected_final_decision,
        "ci_fast_gate": args.ci_fast_gate,
        "required_reason_codes": args.require_reason_code,
        "observed_status": payload.get("status"),
        "observed_final_decision": observed_final_decision,
        "observed_reason_code": observed_reason_code,
        "reason_codes": reason_codes,
        "final_decision": final_decision,
    }

    if args.output_json:
        output_path = Path(args.output_json).resolve()
        output_path.parent.mkdir(parents=True, exist_ok=True)
        output_path.write_text(json.dumps(output, sort_keys=True, indent=2) + "\n", encoding="utf-8")
    status = "ok" if final_decision == GO_DECISION else "fail"
    failed_checks = ",".join(reason_codes) if reason_codes else "none"
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"failed_checks={failed_checks}")
    if args.output_json:
        print(f"report_file={Path(args.output_json).resolve()}")
    return 0 if final_decision == GO_DECISION else 1


def main() -> int:
    args = parse_args()
    if args.command == "generate":
        return generate_bundle(args)
    if args.command == "check":
        return check_bundle(args)
    raise SystemExit("unsupported command")


if __name__ == "__main__":
    raise SystemExit(main())
