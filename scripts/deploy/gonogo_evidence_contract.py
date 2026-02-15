#!/usr/bin/env python3
"""Go/no-go release evidence generator and policy checker."""

from __future__ import annotations

import argparse
from datetime import datetime, timezone
import hashlib
import os
from pathlib import Path
import sys
from typing import Any

SCRIPT_DIR = Path(__file__).resolve().parent
sys.path.insert(0, str(SCRIPT_DIR.parent))
ROOT_DIR = SCRIPT_DIR.parent.parent

from framework.contract_framework import (  # noqa: E402
    ContractError,
    fail,
    load_json,
    parse_int,
    require_keys,
    write_json,
)

SCHEMA_VERSION = "kamn.release.gonogo.v1"
MILESTONE_REVIEW_SCHEMA_VERSION = "kamn.release.milestone-review-bundle.v1"
GO_DECISION = "GO"
NO_GO_DECISION = "NO-GO"
DEFAULT_OPERATOR_RUNBOOK_DOC = ROOT_DIR / "docs/foundation/upgrade-rollback-runbook.md"
REQUIRED_OPERATOR_RUNBOOK_MARKERS = (
    "## Deployment SLO Evidence and Rollback Automation Contract",
    "run_deployment_slo_rollback_lane.sh",
    "check_deployment_slo_rollback_policy.sh",
    "run_deployment_slo_rollback_contract_lane.sh",
    "kamn.deploy.slo-rollback-report.v1",
    "Regression: #944",
)
REQUIRED_EVIDENCE_MARKERS = (
    "ci_fast_gate",
    "ci_deep_lane",
    "rollback_precheck",
    "rollback_trigger_status",
    "approval_quorum",
    "runtime_image_digest",
)

PREFLIGHT_SUMMARY_SCHEMA = "kamn.kolme.local-live-deployment-preflight-summary.v1"
PREFLIGHT_POLICY_SCHEMA = "kamn.kolme.local-live-deployment-preflight-policy-report.v1"
LIVE_BUNDLE_SUMMARY_SCHEMA = "kamn.kolme.local-live-node-validation-bundle-summary.v1"
LIVE_BUNDLE_POLICY_SCHEMA = "kamn.kolme.local-live-node-validation-bundle-policy-report.v1"
GO_NO_GO_GATE_SCHEMA = "kamn.runtime.go-no-go-gate-report.v1"

MILESTONE_ARTIFACT_ARGS = (
    ("deployment_preflight_summary_file", "deployment_preflight_summary_file"),
    ("deployment_preflight_policy_file", "deployment_preflight_policy_file"),
    ("live_node_validation_summary_file", "live_node_validation_summary_file"),
    ("live_node_validation_policy_file", "live_node_validation_policy_file"),
    ("go_no_go_gate_report_file", "go_no_go_gate_report_file"),
)


def _artifact_sha256(path: Path) -> str:
    if not path.is_file():
        return ""
    return hashlib.sha256(path.read_bytes()).hexdigest()


def _resolve_operator_runbook_doc() -> Path:
    override = os.getenv("KAMN_GONOGO_RUNBOOK_DOC_FILE", "").strip()
    if override:
        return Path(override).resolve()
    return DEFAULT_OPERATOR_RUNBOOK_DOC.resolve()


def _operator_runbook_marker_status(runbook_doc: Path) -> tuple[bool, list[str]]:
    if not runbook_doc.is_file():
        return False, list(REQUIRED_OPERATOR_RUNBOOK_MARKERS)

    try:
        runbook_text = runbook_doc.read_text(encoding="utf-8")
    except OSError:
        return False, list(REQUIRED_OPERATOR_RUNBOOK_MARKERS)

    missing_markers = [marker for marker in REQUIRED_OPERATOR_RUNBOOK_MARKERS if marker not in runbook_text]
    return len(missing_markers) == 0, missing_markers


def _optional_milestone_artifact_paths(args: argparse.Namespace) -> dict[str, Path] | None:
    raw_values: dict[str, str] = {}
    for arg_name, field_name in MILESTONE_ARTIFACT_ARGS:
        raw = getattr(args, arg_name, "")
        raw_values[field_name] = raw.strip() if isinstance(raw, str) else ""

    provided = [bool(value) for value in raw_values.values()]
    if any(provided) and not all(provided):
        fail(
            "milestone review artifact arguments must be provided together: "
            "--deployment-preflight-summary-file, --deployment-preflight-policy-file, "
            "--live-node-validation-summary-file, --live-node-validation-policy-file, "
            "--go-no-go-gate-report-file"
        )
    if not any(provided):
        return None

    return {field_name: Path(raw_values[field_name]).resolve() for field_name in raw_values}


def _load_milestone_artifact(
    path: Path,
    reason_codes: list[str],
    missing_reason_code: str,
    invalid_json_reason_code: str,
) -> dict[str, object] | None:
    if not path.is_file():
        reason_codes.append(missing_reason_code)
        return None

    try:
        payload = load_json(path)
    except ContractError:
        reason_codes.append(invalid_json_reason_code)
        return None

    if not isinstance(payload, dict):
        reason_codes.append(invalid_json_reason_code)
        return None
    return payload


def _build_milestone_review_bundle(artifact_paths: dict[str, Path]) -> dict[str, Any]:
    reason_codes: list[str] = []
    operator_runbook_doc = _resolve_operator_runbook_doc()
    operator_runbook_markers_present, operator_runbook_missing_markers = _operator_runbook_marker_status(
        operator_runbook_doc
    )
    if not operator_runbook_doc.is_file():
        reason_codes.append("milestone_review_operator_runbook_missing")
    elif not operator_runbook_markers_present:
        reason_codes.append("milestone_review_operator_runbook_markers_missing")

    preflight_summary = _load_milestone_artifact(
        artifact_paths["deployment_preflight_summary_file"],
        reason_codes,
        "milestone_review_deployment_preflight_summary_missing",
        "milestone_review_deployment_preflight_summary_invalid_json",
    )
    preflight_policy = _load_milestone_artifact(
        artifact_paths["deployment_preflight_policy_file"],
        reason_codes,
        "milestone_review_deployment_preflight_policy_missing",
        "milestone_review_deployment_preflight_policy_invalid_json",
    )
    live_summary = _load_milestone_artifact(
        artifact_paths["live_node_validation_summary_file"],
        reason_codes,
        "milestone_review_live_node_validation_summary_missing",
        "milestone_review_live_node_validation_summary_invalid_json",
    )
    live_policy = _load_milestone_artifact(
        artifact_paths["live_node_validation_policy_file"],
        reason_codes,
        "milestone_review_live_node_validation_policy_missing",
        "milestone_review_live_node_validation_policy_invalid_json",
    )
    gate_report = _load_milestone_artifact(
        artifact_paths["go_no_go_gate_report_file"],
        reason_codes,
        "milestone_review_go_no_go_gate_report_missing",
        "milestone_review_go_no_go_gate_report_invalid_json",
    )

    preflight_status = ""
    preflight_scope = ""
    if preflight_summary is not None:
        if preflight_summary.get("schema_version") != PREFLIGHT_SUMMARY_SCHEMA:
            reason_codes.append("milestone_review_deployment_preflight_summary_schema_mismatch")
        preflight_status = str(preflight_summary.get("status", ""))
        if preflight_status != "ok":
            reason_codes.append("milestone_review_deployment_preflight_summary_status_mismatch")
        contracts = preflight_summary.get("contracts")
        if isinstance(contracts, dict):
            preflight_scope = str(contracts.get("ci_fast_gate_scope", ""))
        if preflight_scope != "ci-fast-gate":
            reason_codes.append("milestone_review_deployment_preflight_scope_mismatch")

    preflight_policy_final_decision = ""
    if preflight_policy is not None:
        if preflight_policy.get("schema_version") != PREFLIGHT_POLICY_SCHEMA:
            reason_codes.append("milestone_review_deployment_preflight_policy_schema_mismatch")
        preflight_policy_final_decision = str(preflight_policy.get("final_decision", ""))
        if preflight_policy_final_decision != GO_DECISION:
            reason_codes.append("milestone_review_deployment_preflight_policy_final_decision_mismatch")

    live_status = ""
    live_scope = ""
    live_runtime_provider = ""
    live_rollback_recovery_lineage_required = False
    live_rollback_evidence_present = False
    live_recovery_evidence_present = False
    if live_summary is not None:
        if live_summary.get("schema_version") != LIVE_BUNDLE_SUMMARY_SCHEMA:
            reason_codes.append("milestone_review_live_node_validation_summary_schema_mismatch")
        live_status = str(live_summary.get("status", ""))
        if live_status != "ok":
            reason_codes.append("milestone_review_live_node_validation_summary_status_mismatch")
        contracts = live_summary.get("contracts")
        if isinstance(contracts, dict):
            live_scope = str(contracts.get("ci_fast_gate_scope", ""))
            live_runtime_provider = str(contracts.get("runtime_provider_client_contract", ""))
            live_rollback_recovery_lineage_required = (
                contracts.get("rollback_recovery_artifact_lineage_required") is True
            )
        if live_scope != "local-only":
            reason_codes.append("milestone_review_live_node_validation_scope_mismatch")
        if live_runtime_provider != "KolmeRuntimeCommitLiveProvider":
            reason_codes.append("milestone_review_live_node_validation_runtime_provider_mismatch")
        if not live_rollback_recovery_lineage_required:
            reason_codes.append("milestone_review_live_node_validation_lineage_contract_mismatch")

        rollback_evidence_file = str(live_summary.get("rollback_evidence_file", "")).strip()
        recovery_evidence_file = str(live_summary.get("recovery_evidence_file", "")).strip()
        artifact_list = live_summary.get("artifact_paths")
        artifact_paths_list: list[str] = []
        if isinstance(artifact_list, list):
            artifact_paths_list = [
                entry.strip() for entry in artifact_list if isinstance(entry, str) and entry.strip()
            ]
        if not artifact_paths_list:
            reason_codes.append("milestone_review_live_node_validation_artifact_paths_missing")
        live_rollback_evidence_present = bool(
            rollback_evidence_file and rollback_evidence_file in artifact_paths_list
        )
        live_recovery_evidence_present = bool(
            recovery_evidence_file and recovery_evidence_file in artifact_paths_list
        )
        if not live_rollback_evidence_present:
            reason_codes.append("milestone_review_live_node_validation_rollback_lineage_missing")
        if not live_recovery_evidence_present:
            reason_codes.append("milestone_review_live_node_validation_recovery_lineage_missing")

    live_policy_final_decision = ""
    if live_policy is not None:
        if live_policy.get("schema_version") != LIVE_BUNDLE_POLICY_SCHEMA:
            reason_codes.append("milestone_review_live_node_validation_policy_schema_mismatch")
        live_policy_final_decision = str(live_policy.get("final_decision", ""))
        if live_policy_final_decision != GO_DECISION:
            reason_codes.append("milestone_review_live_node_validation_policy_final_decision_mismatch")

    gate_status = ""
    gate_final_decision = ""
    if gate_report is not None:
        if gate_report.get("schema_version") != GO_NO_GO_GATE_SCHEMA:
            reason_codes.append("milestone_review_go_no_go_gate_schema_mismatch")
        gate_status = str(gate_report.get("status", ""))
        if gate_status != "pass":
            reason_codes.append("milestone_review_go_no_go_gate_status_mismatch")
        gate_final_decision = str(gate_report.get("final_decision", ""))
        if gate_final_decision != GO_DECISION:
            reason_codes.append("milestone_review_go_no_go_gate_final_decision_mismatch")

    reason_codes = sorted(set(reason_codes))
    final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION
    lineage_status = "verified" if final_decision == GO_DECISION else "fail-closed"

    artifacts: dict[str, str] = {}
    for field_name, path in artifact_paths.items():
        artifacts[field_name] = str(path)
        artifacts[f"{field_name[:-5]}sha256"] = _artifact_sha256(path)
    artifacts["operator_runbook_doc_file"] = str(operator_runbook_doc)
    artifacts["operator_runbook_doc_sha256"] = _artifact_sha256(operator_runbook_doc)

    return {
        "schema_version": MILESTONE_REVIEW_SCHEMA_VERSION,
        "final_decision": final_decision,
        "lineage_status": lineage_status,
        "reason_codes": reason_codes,
        "artifacts": artifacts,
        "observed": {
            "deployment_preflight_summary_status": preflight_status,
            "deployment_preflight_policy_final_decision": preflight_policy_final_decision,
            "live_node_validation_summary_status": live_status,
            "live_node_validation_policy_final_decision": live_policy_final_decision,
            "go_no_go_gate_status": gate_status,
            "go_no_go_gate_final_decision": gate_final_decision,
            "deployment_preflight_contract_scope": preflight_scope,
            "live_node_validation_contract_scope": live_scope,
            "live_node_validation_runtime_provider_client_contract": live_runtime_provider,
            "live_node_validation_lineage_contract_present": live_rollback_recovery_lineage_required,
            "live_node_validation_rollback_lineage_present": live_rollback_evidence_present,
            "live_node_validation_recovery_lineage_present": live_recovery_evidence_present,
            "operator_runbook_markers_present": operator_runbook_markers_present,
            "operator_runbook_missing_markers": operator_runbook_missing_markers,
        },
        "contracts": {
            "linked_artifact_lineage_required": True,
            "operator_runbook_markers_required": True,
            "operator_runbook_required_markers": list(REQUIRED_OPERATOR_RUNBOOK_MARKERS),
            "deployment_preflight_scope_required": "ci-fast-gate",
            "live_bundle_scope_required": "local-only",
            "live_bundle_runtime_provider_client_required": "KolmeRuntimeCommitLiveProvider",
            "live_bundle_rollback_recovery_lineage_required": True,
            "deployment_preflight_policy_final_decision_required": GO_DECISION,
            "live_bundle_policy_final_decision_required": GO_DECISION,
            "go_no_go_gate_status_required": "pass",
            "go_no_go_gate_final_decision_required": GO_DECISION,
        },
    }


def _validated_expected_milestone_bundle(
    payload: dict[str, object],
) -> tuple[dict[str, Any], str]:
    milestone_review_bundle = payload.get("milestone_review_bundle")
    if not isinstance(milestone_review_bundle, dict):
        fail("bundle field 'milestone_review_bundle' must be an object when provided")

    require_keys(
        milestone_review_bundle,
        (
            "schema_version",
            "final_decision",
            "lineage_status",
            "reason_codes",
            "artifacts",
            "observed",
            "contracts",
        ),
    )

    artifacts = milestone_review_bundle["artifacts"]
    if not isinstance(artifacts, dict):
        fail("milestone_review_bundle.artifacts must be an object")

    artifact_paths: dict[str, Path] = {}
    for _, field_name in MILESTONE_ARTIFACT_ARGS:
        artifact_path = artifacts.get(field_name)
        if not isinstance(artifact_path, str) or not artifact_path.strip():
            fail(f"milestone_review_bundle.artifacts.{field_name} must be a non-empty string")
        artifact_paths[field_name] = Path(artifact_path).resolve()

    expected_bundle = _build_milestone_review_bundle(artifact_paths)
    if milestone_review_bundle != expected_bundle:
        fail(
            "milestone review bundle lineage mismatch: "
            "expected deterministic aggregate artifact markers and decision surface"
        )

    expected_decision = expected_bundle["final_decision"]
    if not isinstance(expected_decision, str):
        fail("milestone review bundle final decision must be a string")
    return expected_bundle, expected_decision


def generate_bundle(args: argparse.Namespace) -> int:
    required_values = (
        args.output_file,
        args.release_candidate,
        args.schema_target_version,
        args.runtime_image_digest,
        args.ci_fast_gate,
        args.ci_deep_lane,
        args.rollback_precheck,
        args.rollback_trigger_status,
        args.required_approvals,
        args.received_approvals,
    )
    if any(value is None or value == "" for value in required_values):
        fail("all bundle arguments are required")

    ci_fast_gate = args.ci_fast_gate
    ci_deep_lane = args.ci_deep_lane
    rollback_precheck = args.rollback_precheck
    for field_name, value in (
        ("ci-fast-gate", ci_fast_gate),
        ("ci-deep-lane", ci_deep_lane),
        ("rollback-precheck", rollback_precheck),
    ):
        if value not in {"PASS", "FAIL"}:
            fail(f"{field_name} must be PASS or FAIL")

    rollback_trigger_status = args.rollback_trigger_status
    if rollback_trigger_status not in {"CLEAR", "TRIGGERED"}:
        fail("rollback-trigger-status must be CLEAR or TRIGGERED")

    required_approvals = parse_int("required-approvals", args.required_approvals)
    received_approvals = parse_int("received-approvals", args.received_approvals)
    if required_approvals < 1:
        fail("required-approvals must be >= 1")
    if received_approvals < 0:
        fail("received-approvals must be >= 0")

    expected_go = (
        ci_fast_gate == "PASS"
        and ci_deep_lane == "PASS"
        and rollback_precheck == "PASS"
        and rollback_trigger_status == "CLEAR"
        and received_approvals >= required_approvals
    )

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "release_candidate": args.release_candidate,
        "schema_target_version": args.schema_target_version,
        "runtime_image_digest": args.runtime_image_digest,
        "evidence_markers": list(REQUIRED_EVIDENCE_MARKERS),
        "gates": {
            "ci_fast_gate": ci_fast_gate,
            "ci_deep_lane": ci_deep_lane,
            "rollback_precheck": rollback_precheck,
        },
        "rollback_trigger_status": rollback_trigger_status,
        "approvals": {
            "required": required_approvals,
            "received": received_approvals,
        },
    }

    milestone_review_bundle = None
    milestone_review_artifacts = _optional_milestone_artifact_paths(args)
    if milestone_review_artifacts is not None:
        milestone_review_bundle = _build_milestone_review_bundle(milestone_review_artifacts)
        payload["milestone_review_bundle"] = milestone_review_bundle
        expected_go = expected_go and milestone_review_bundle["final_decision"] == GO_DECISION

    final_decision = GO_DECISION if expected_go else NO_GO_DECISION
    payload["final_decision"] = final_decision

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    if milestone_review_bundle is not None:
        print(f"milestone_review_final_decision={milestone_review_bundle['final_decision']}")
    print(f"final_decision={final_decision}")
    return 0


def check_bundle(args: argparse.Namespace) -> int:
    if not args.bundle_file:
        fail("--bundle-file is required")

    bundle_path = Path(args.bundle_file)
    if not bundle_path.is_file():
        fail(f"bundle file not found: {bundle_path}")

    payload = load_json(bundle_path)
    require_keys(
        payload,
        (
            "schema_version",
            "generated_at",
            "release_candidate",
            "schema_target_version",
            "runtime_image_digest",
            "evidence_markers",
            "gates",
            "rollback_trigger_status",
            "approvals",
            "final_decision",
        ),
    )

    gates = payload["gates"]
    if not isinstance(gates, dict):
        fail("bundle field 'gates' must be an object")

    for gate_name in ("ci_fast_gate", "ci_deep_lane", "rollback_precheck"):
        if gate_name not in gates:
            fail(f"missing gate field: {gate_name}")
        if gates[gate_name] not in {"PASS", "FAIL"}:
            fail(f"gate '{gate_name}' must be PASS or FAIL")

    rollback_trigger_status = payload["rollback_trigger_status"]
    if rollback_trigger_status not in {"CLEAR", "TRIGGERED"}:
        fail("rollback_trigger_status must be CLEAR or TRIGGERED")

    evidence_markers = payload["evidence_markers"]
    if not isinstance(evidence_markers, list):
        fail("bundle field 'evidence_markers' must be an array")
    if any(not isinstance(marker, str) or marker == "" for marker in evidence_markers):
        fail("evidence_markers entries must be non-empty strings")
    missing_required_markers = [
        marker for marker in REQUIRED_EVIDENCE_MARKERS if marker not in evidence_markers
    ]
    if missing_required_markers:
        fail(
            "missing required evidence markers: "
            + ",".join(sorted(set(missing_required_markers)))
        )

    approvals = payload["approvals"]
    if not isinstance(approvals, dict):
        fail("bundle field 'approvals' must be an object")
    if "required" not in approvals:
        fail("missing approvals field: required")
    if "received" not in approvals:
        fail("missing approvals field: received")

    required_approvals = approvals["required"]
    received_approvals = approvals["received"]
    if not isinstance(required_approvals, int):
        fail("approvals.required must be an integer")
    if not isinstance(received_approvals, int):
        fail("approvals.received must be an integer")
    if required_approvals < 1:
        fail("approvals.required must be >= 1")
    if received_approvals < 0:
        fail("approvals.received must be >= 0")

    expected_go = (
        gates["ci_fast_gate"] == "PASS"
        and gates["ci_deep_lane"] == "PASS"
        and gates["rollback_precheck"] == "PASS"
        and rollback_trigger_status == "CLEAR"
        and received_approvals >= required_approvals
    )

    milestone_decision = GO_DECISION
    if "milestone_review_bundle" in payload:
        _, milestone_decision = _validated_expected_milestone_bundle(payload)
        expected_go = expected_go and milestone_decision == GO_DECISION

    expected_decision = GO_DECISION if expected_go else NO_GO_DECISION
    actual_decision = payload["final_decision"]
    if actual_decision not in {GO_DECISION, NO_GO_DECISION}:
        fail("final_decision must be GO or NO-GO")
    if actual_decision != expected_decision:
        fail(
            "policy decision mismatch: "
            f"expected final_decision={expected_decision}, found {actual_decision}"
        )

    print("status=ok")
    print(f"bundle_file={bundle_path}")
    if "milestone_review_bundle" in payload:
        print(f"milestone_review_final_decision={milestone_decision}")
    print(f"final_decision={actual_decision}")
    print(f"required_approvals={required_approvals}")
    print(f"received_approvals={received_approvals}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        description="Go/no-go release evidence contract utilities (generate/check)."
    )
    subparsers = parser.add_subparsers(dest="command", required=True)

    generate = subparsers.add_parser("generate")
    generate.add_argument("--output-file")
    generate.add_argument("--release-candidate")
    generate.add_argument("--schema-target-version")
    generate.add_argument("--runtime-image-digest")
    generate.add_argument("--ci-fast-gate")
    generate.add_argument("--ci-deep-lane")
    generate.add_argument("--rollback-precheck")
    generate.add_argument("--rollback-trigger-status")
    generate.add_argument("--required-approvals")
    generate.add_argument("--received-approvals")
    generate.add_argument("--deployment-preflight-summary-file", default="")
    generate.add_argument("--deployment-preflight-policy-file", default="")
    generate.add_argument("--live-node-validation-summary-file", default="")
    generate.add_argument("--live-node-validation-policy-file", default="")
    generate.add_argument("--go-no-go-gate-report-file", default="")
    generate.set_defaults(handler=generate_bundle)

    check = subparsers.add_parser("check")
    check.add_argument("--bundle-file")
    check.set_defaults(handler=check_bundle)

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
