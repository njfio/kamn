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
LIVE_GONOGO_REASON_TAXONOMY_VERSION = "kamn.release.gonogo-live-evidence-convergence-reason-taxonomy.v1"
TLS_EVIDENCE_GATE_SCHEMA_VERSION = "kamn.release.gonogo-tls-evidence-gate.v1"
AUDIT_INTEGRITY_GATE_SCHEMA_VERSION = "kamn.release.gonogo-audit-integrity-gate.v1"
SLO_POLICY_GATE_SCHEMA_VERSION = "kamn.release.gonogo-slo-policy-gate.v1"
INCIDENT_READINESS_GATE_SCHEMA_VERSION = "kamn.release.gonogo-incident-readiness-gate.v1"
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
TLS_EVIDENCE_SCHEMA_VERSION = "kamn.ci.kamn-core-live-https-dependency-posture-report.v1"
TLS_EVIDENCE_SOURCE_REASON_TAXONOMY_VERSION = (
    "kamn.ci.kamn-core-live-https-dependency-posture-reason-taxonomy.v1"
)
TLS_EVIDENCE_GATE_REASON_TAXONOMY_VERSION = (
    "kamn.release.gonogo-tls-evidence-convergence-reason-taxonomy.v1"
)
DEFAULT_TLS_EVIDENCE_MAX_AGE_SECONDS = 1800
AUDIT_INTEGRITY_SOURCE_SCHEMA_VERSION = (
    "kamn.runtime.sqlite-crash-recovery-live-policy-report.v1"
)
AUDIT_INTEGRITY_SOURCE_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.durability-governance-reason-taxonomy.v1"
)
AUDIT_INTEGRITY_SOURCE_REASON_CODES_CSV = (
    "crash_recovery_promotion_stalled,audit_trail_parity_mismatch,"
    "ci_local_promotion_budget_boundary_exceeded"
)
AUDIT_INTEGRITY_GATE_REASON_TAXONOMY_VERSION = (
    "kamn.release.gonogo-audit-integrity-convergence-reason-taxonomy.v1"
)
DEFAULT_AUDIT_INTEGRITY_MAX_AGE_SECONDS = 1800
SLO_POLICY_SOURCE_SCHEMA_VERSION = "kamn.deploy.slo-rollback-report.v1"
SLO_POLICY_SOURCE_REASON_KEY = "deployment_slo_rollback_reason_codes:GO:v1"
SLO_POLICY_GATE_REASON_TAXONOMY_VERSION = (
    "kamn.release.gonogo-slo-threshold-convergence-reason-taxonomy.v1"
)
DEFAULT_SLO_POLICY_MAX_AGE_SECONDS = 1800
INCIDENT_READINESS_SOURCE_SCHEMA_VERSION = "kamn.release.staging-rehearsal.v1"
INCIDENT_READINESS_SOURCE_OUTPUT_CONTRACT_VERSION = (
    "kamn.release.staging-rehearsal-output-contract.v1"
)
INCIDENT_READINESS_SOURCE_REASON_TAXONOMY_SCHEMA_VERSION = (
    "kamn.release.staging-rehearsal-reason-taxonomy.v1"
)
INCIDENT_READINESS_SOURCE_NORMALIZED_EVIDENCE_SCHEMA_VERSION = (
    "kamn.release.staging-rehearsal-evidence-normalization.v1"
)
INCIDENT_READINESS_SOURCE_STAGED_SIGNOFF_SCHEMA_VERSION = (
    "kamn.release.staged-rehearsal-signoff.v1"
)
INCIDENT_READINESS_SOURCE_REASON_CODES = ["all rehearsal gates satisfied"]
INCIDENT_READINESS_GATE_REASON_TAXONOMY_VERSION = (
    "kamn.release.gonogo-incident-readiness-convergence-reason-taxonomy.v1"
)
DEFAULT_INCIDENT_READINESS_MAX_AGE_SECONDS = 1800

PREFLIGHT_SUMMARY_SCHEMA = "kamn.kolme.local-live-deployment-preflight-summary.v1"
PREFLIGHT_POLICY_SCHEMA = "kamn.kolme.local-live-deployment-preflight-policy-report.v1"
LIVE_BUNDLE_SUMMARY_SCHEMA = "kamn.kolme.local-live-node-validation-bundle-summary.v1"
LIVE_BUNDLE_POLICY_SCHEMA = "kamn.kolme.local-live-node-validation-bundle-policy-report.v1"
GO_NO_GO_GATE_SCHEMA = "kamn.runtime.go-no-go-gate-report.v1"
COMBINED_REASON_TAXONOMY_VERSION = "kamn.runtime.local-full-stack-integration-reason-taxonomy.v1"
COMBINED_TRANSPORT_REASON_CODES = [  # deterministic singleton today; model as list for schema stability.
    "fork_choice_stale_block_height",
]
ALLOWED_COMBINED_KOLME_REASON_CODES = {
    "not_run",
    "live_runtime_integration_passed",
}
KOLME_RUNTIME_COMMIT_FAILURE_TAXONOMY_VERSION = "v1"
KOLME_RUNTIME_COMMIT_PROFILE = "real-node-non-synthetic-v1"
KOLME_RUNTIME_COMMIT_PROFILE_VERSION = "v1"

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


def _parse_utc_timestamp(value: object, field_name: str) -> datetime:
    if not isinstance(value, str) or not value.strip():
        fail(f"{field_name} must be a non-empty UTC timestamp")
    try:
        parsed = datetime.strptime(value, "%Y-%m-%dT%H:%M:%SZ")
    except ValueError:
        fail(f"{field_name} must use UTC timestamp format YYYY-MM-DDTHH:MM:SSZ")
    return parsed.replace(tzinfo=timezone.utc)


def _optional_tls_evidence_gate_inputs(args: argparse.Namespace) -> tuple[Path, int] | None:
    raw_report_file = getattr(args, "tls_evidence_report_file", "")
    raw_max_age = getattr(args, "tls_evidence_max_age_seconds", "")

    report_file = raw_report_file.strip() if isinstance(raw_report_file, str) else ""
    max_age_value = raw_max_age.strip() if isinstance(raw_max_age, str) else ""

    if not report_file and not max_age_value:
        return None
    if not report_file:
        fail(
            "--tls-evidence-report-file is required when "
            "--tls-evidence-max-age-seconds is provided"
        )

    if not max_age_value:
        max_age_seconds = DEFAULT_TLS_EVIDENCE_MAX_AGE_SECONDS
    else:
        max_age_seconds = parse_int("tls-evidence-max-age-seconds", max_age_value)
        if max_age_seconds < 1:
            fail("tls-evidence-max-age-seconds must be >= 1")

    return Path(report_file).resolve(), max_age_seconds


def _build_tls_evidence_gate(
    report_path: Path, max_age_seconds: int, reference_time: datetime
) -> dict[str, Any]:
    reason_codes: list[str] = []
    report_payload: dict[str, Any] | None = None
    report_schema_version = ""
    report_status = ""
    report_reason_taxonomy_version = ""
    report_reason_codes_csv = ""
    report_reason_codes_value = ""
    report_reason_codes: list[str] = []
    report_mtime_utc = ""
    report_age_seconds = -1

    if max_age_seconds < 1:
        reason_codes.append("gonogo_tls_evidence_max_age_invalid")

    if not report_path.is_file():
        reason_codes.append("gonogo_tls_evidence_file_missing")
    else:
        mtime_utc = datetime.fromtimestamp(report_path.stat().st_mtime, tz=timezone.utc)
        report_mtime_utc = mtime_utc.strftime("%Y-%m-%dT%H:%M:%SZ")
        report_age_seconds = max(
            0, int((reference_time - mtime_utc).total_seconds())
        )
        if max_age_seconds >= 1 and report_age_seconds > max_age_seconds:
            reason_codes.append("gonogo_tls_evidence_freshness_window_exceeded")

        try:
            payload = load_json(report_path)
        except ContractError:
            reason_codes.append("gonogo_tls_evidence_invalid_json")
        else:
            if not isinstance(payload, dict):
                reason_codes.append("gonogo_tls_evidence_invalid_json")
            else:
                report_payload = payload

    if report_payload is not None:
        report_schema_version = str(report_payload.get("schema_version", ""))
        report_status = str(report_payload.get("status", ""))
        report_reason_taxonomy_version = str(
            report_payload.get("reason_taxonomy_version", "")
        )
        report_reason_codes_csv = str(report_payload.get("reason_codes_csv", ""))
        report_reason_codes_value = str(report_payload.get("reason_codes_value", ""))
        raw_reason_codes = report_payload.get("reason_codes")
        if isinstance(raw_reason_codes, list):
            report_reason_codes = [
                value for value in raw_reason_codes if isinstance(value, str) and value
            ]
        else:
            reason_codes.append("gonogo_tls_evidence_reason_codes_invalid")

        if report_schema_version != TLS_EVIDENCE_SCHEMA_VERSION:
            reason_codes.append("gonogo_tls_evidence_schema_mismatch")
        if report_reason_taxonomy_version != TLS_EVIDENCE_SOURCE_REASON_TAXONOMY_VERSION:
            reason_codes.append("gonogo_tls_evidence_reason_taxonomy_version_mismatch")
        if report_status != "pass":
            reason_codes.append("gonogo_tls_evidence_status_not_pass")
        if report_status == "pass" and report_reason_codes != ["none"]:
            reason_codes.append("gonogo_tls_evidence_reason_codes_invalid")

    reason_codes = sorted(set(reason_codes))
    final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)
    gate_status = "verified" if final_decision == GO_DECISION else "fail-closed"

    return {
        "schema_version": TLS_EVIDENCE_GATE_SCHEMA_VERSION,
        "reason_taxonomy_version": TLS_EVIDENCE_GATE_REASON_TAXONOMY_VERSION,
        "final_decision": final_decision,
        "status": gate_status,
        "reason_codes": reason_codes,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes_value": reason_codes_csv,
        "artifacts": {
            "tls_evidence_report_file": str(report_path),
            "tls_evidence_report_sha256": _artifact_sha256(report_path),
        },
        "observed": {
            "tls_evidence_report_schema_version": report_schema_version,
            "tls_evidence_report_status": report_status,
            "tls_evidence_report_reason_taxonomy_version": report_reason_taxonomy_version,
            "tls_evidence_report_reason_codes_csv": report_reason_codes_csv,
            "tls_evidence_report_reason_codes_value": report_reason_codes_value,
            "tls_evidence_report_reason_codes": report_reason_codes,
            "tls_evidence_report_mtime_utc": report_mtime_utc,
            "tls_evidence_report_age_seconds": report_age_seconds,
        },
        "contracts": {
            "tls_evidence_report_schema_version_required": TLS_EVIDENCE_SCHEMA_VERSION,
            "tls_evidence_report_reason_taxonomy_version_required": (
                TLS_EVIDENCE_SOURCE_REASON_TAXONOMY_VERSION
            ),
            "tls_evidence_report_status_required": "pass",
            "tls_evidence_report_reason_codes_required": ["none"],
            "tls_evidence_max_age_seconds_required": max_age_seconds,
        },
    }


def _optional_audit_integrity_gate_inputs(args: argparse.Namespace) -> tuple[Path, int] | None:
    raw_report_file = getattr(args, "audit_integrity_report_file", "")
    raw_max_age = getattr(args, "audit_integrity_max_age_seconds", "")

    report_file = raw_report_file.strip() if isinstance(raw_report_file, str) else ""
    max_age_value = raw_max_age.strip() if isinstance(raw_max_age, str) else ""

    if not report_file and not max_age_value:
        return None
    if not report_file:
        fail(
            "--audit-integrity-report-file is required when "
            "--audit-integrity-max-age-seconds is provided"
        )

    if not max_age_value:
        max_age_seconds = DEFAULT_AUDIT_INTEGRITY_MAX_AGE_SECONDS
    else:
        max_age_seconds = parse_int("audit-integrity-max-age-seconds", max_age_value)
        if max_age_seconds < 1:
            fail("audit-integrity-max-age-seconds must be >= 1")

    return Path(report_file).resolve(), max_age_seconds


def _build_audit_integrity_gate(
    report_path: Path, max_age_seconds: int, reference_time: datetime
) -> dict[str, Any]:
    reason_codes: list[str] = []
    report_payload: dict[str, Any] | None = None
    report_schema_version = ""
    report_status = ""
    report_final_decision = ""
    report_policy_status = ""
    report_reason_taxonomy_version = ""
    report_reason_codes_csv = ""
    report_mtime_utc = ""
    report_age_seconds = -1

    if max_age_seconds < 1:
        reason_codes.append("gonogo_audit_integrity_max_age_invalid")

    if not report_path.is_file():
        reason_codes.append("gonogo_audit_integrity_file_missing")
    else:
        mtime_utc = datetime.fromtimestamp(report_path.stat().st_mtime, tz=timezone.utc)
        report_mtime_utc = mtime_utc.strftime("%Y-%m-%dT%H:%M:%SZ")
        report_age_seconds = max(
            0, int((reference_time - mtime_utc).total_seconds())
        )
        if max_age_seconds >= 1 and report_age_seconds > max_age_seconds:
            reason_codes.append("gonogo_audit_integrity_freshness_window_exceeded")

        try:
            payload = load_json(report_path)
        except ContractError:
            reason_codes.append("gonogo_audit_integrity_invalid_json")
        else:
            if not isinstance(payload, dict):
                reason_codes.append("gonogo_audit_integrity_invalid_json")
            else:
                report_payload = payload

    if report_payload is not None:
        report_schema_version = str(report_payload.get("schema_version", ""))
        report_status = str(report_payload.get("status", ""))
        report_final_decision = str(report_payload.get("final_decision", ""))
        report_policy_status = str(
            report_payload.get("sqlite_crash_recovery_policy_status", "")
        )
        report_reason_taxonomy_version = str(
            report_payload.get("durability_governance_reason_taxonomy_version", "")
        )
        report_reason_codes_csv = str(
            report_payload.get("durability_governance_reason_codes_csv", "")
        )

        if report_schema_version != AUDIT_INTEGRITY_SOURCE_SCHEMA_VERSION:
            reason_codes.append("gonogo_audit_integrity_schema_mismatch")
        if report_status != "ok":
            reason_codes.append("gonogo_audit_integrity_status_not_ok")
        if report_final_decision != GO_DECISION:
            reason_codes.append("gonogo_audit_integrity_final_decision_not_go")
        if report_policy_status != "verified":
            reason_codes.append("gonogo_audit_integrity_policy_status_not_verified")
        if (
            report_reason_taxonomy_version
            != AUDIT_INTEGRITY_SOURCE_REASON_TAXONOMY_VERSION
        ):
            reason_codes.append(
                "gonogo_audit_integrity_reason_taxonomy_version_mismatch"
            )
        if report_reason_codes_csv != AUDIT_INTEGRITY_SOURCE_REASON_CODES_CSV:
            reason_codes.append("gonogo_audit_integrity_reason_codes_csv_mismatch")

    reason_codes = sorted(set(reason_codes))
    final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION
    gate_status = "verified" if final_decision == GO_DECISION else "fail-closed"
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

    return {
        "schema_version": AUDIT_INTEGRITY_GATE_SCHEMA_VERSION,
        "reason_taxonomy_version": AUDIT_INTEGRITY_GATE_REASON_TAXONOMY_VERSION,
        "final_decision": final_decision,
        "status": gate_status,
        "reason_codes": reason_codes,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes_value": reason_codes_csv,
        "artifacts": {
            "audit_integrity_report_file": str(report_path),
            "audit_integrity_report_sha256": _artifact_sha256(report_path),
        },
        "observed": {
            "audit_integrity_report_schema_version": report_schema_version,
            "audit_integrity_report_status": report_status,
            "audit_integrity_report_final_decision": report_final_decision,
            "audit_integrity_report_policy_status": report_policy_status,
            "audit_integrity_report_reason_taxonomy_version": (
                report_reason_taxonomy_version
            ),
            "audit_integrity_report_reason_codes_csv": report_reason_codes_csv,
            "audit_integrity_report_mtime_utc": report_mtime_utc,
            "audit_integrity_report_age_seconds": report_age_seconds,
        },
        "contracts": {
            "audit_integrity_report_schema_version_required": (
                AUDIT_INTEGRITY_SOURCE_SCHEMA_VERSION
            ),
            "audit_integrity_report_status_required": "ok",
            "audit_integrity_report_final_decision_required": GO_DECISION,
            "audit_integrity_report_policy_status_required": "verified",
            "audit_integrity_report_reason_taxonomy_version_required": (
                AUDIT_INTEGRITY_SOURCE_REASON_TAXONOMY_VERSION
            ),
            "audit_integrity_report_reason_codes_csv_required": (
                AUDIT_INTEGRITY_SOURCE_REASON_CODES_CSV
            ),
            "audit_integrity_max_age_seconds_required": max_age_seconds,
        },
    }


def _optional_slo_policy_gate_inputs(args: argparse.Namespace) -> tuple[Path, int] | None:
    raw_report_file = getattr(args, "slo_policy_report_file", "")
    raw_max_age = getattr(args, "slo_policy_max_age_seconds", "")

    report_file = raw_report_file.strip() if isinstance(raw_report_file, str) else ""
    max_age_value = raw_max_age.strip() if isinstance(raw_max_age, str) else ""

    if not report_file and not max_age_value:
        return None
    if not report_file:
        fail(
            "--slo-policy-report-file is required when "
            "--slo-policy-max-age-seconds is provided"
        )

    if not max_age_value:
        max_age_seconds = DEFAULT_SLO_POLICY_MAX_AGE_SECONDS
    else:
        max_age_seconds = parse_int("slo-policy-max-age-seconds", max_age_value)
        if max_age_seconds < 1:
            fail("slo-policy-max-age-seconds must be >= 1")

    return Path(report_file).resolve(), max_age_seconds


def _build_slo_policy_gate(
    report_path: Path, max_age_seconds: int, reference_time: datetime
) -> dict[str, Any]:
    reason_codes: list[str] = []
    report_payload: dict[str, Any] | None = None
    report_schema_version = ""
    report_status = ""
    report_final_decision = ""
    report_reason_key = ""
    report_reason_codes: list[str] = []
    report_mtime_utc = ""
    report_age_seconds = -1

    if max_age_seconds < 1:
        reason_codes.append("gonogo_slo_policy_max_age_invalid")

    if not report_path.is_file():
        reason_codes.append("gonogo_slo_policy_file_missing")
    else:
        mtime_utc = datetime.fromtimestamp(report_path.stat().st_mtime, tz=timezone.utc)
        report_mtime_utc = mtime_utc.strftime("%Y-%m-%dT%H:%M:%SZ")
        report_age_seconds = max(
            0, int((reference_time - mtime_utc).total_seconds())
        )
        if max_age_seconds >= 1 and report_age_seconds > max_age_seconds:
            reason_codes.append("gonogo_slo_policy_freshness_window_exceeded")

        try:
            payload = load_json(report_path)
        except ContractError:
            reason_codes.append("gonogo_slo_policy_invalid_json")
        else:
            if not isinstance(payload, dict):
                reason_codes.append("gonogo_slo_policy_invalid_json")
            else:
                report_payload = payload

    if report_payload is not None:
        report_schema_version = str(report_payload.get("schema_version", ""))
        report_status = str(report_payload.get("status", ""))
        report_final_decision = str(report_payload.get("final_decision", ""))
        report_reason_key = str(report_payload.get("reason_key", ""))

        raw_reason_codes = report_payload.get("reason_codes")
        if isinstance(raw_reason_codes, list):
            report_reason_codes = [
                value for value in raw_reason_codes if isinstance(value, str) and value
            ]
        else:
            reason_codes.append("gonogo_slo_policy_reason_codes_not_empty")

        if report_schema_version != SLO_POLICY_SOURCE_SCHEMA_VERSION:
            reason_codes.append("gonogo_slo_policy_schema_mismatch")
        if report_status != "pass":
            reason_codes.append("gonogo_slo_policy_status_not_pass")
        if report_final_decision != GO_DECISION:
            reason_codes.append("gonogo_slo_policy_final_decision_not_go")
        if report_reason_key != SLO_POLICY_SOURCE_REASON_KEY:
            reason_codes.append("gonogo_slo_policy_reason_key_mismatch")
        if report_reason_codes:
            reason_codes.append("gonogo_slo_policy_reason_codes_not_empty")

    reason_codes = sorted(set(reason_codes))
    final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION
    gate_status = "verified" if final_decision == GO_DECISION else "fail-closed"
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

    return {
        "schema_version": SLO_POLICY_GATE_SCHEMA_VERSION,
        "reason_taxonomy_version": SLO_POLICY_GATE_REASON_TAXONOMY_VERSION,
        "final_decision": final_decision,
        "status": gate_status,
        "reason_codes": reason_codes,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes_value": reason_codes_csv,
        "artifacts": {
            "slo_policy_report_file": str(report_path),
            "slo_policy_report_sha256": _artifact_sha256(report_path),
        },
        "observed": {
            "slo_policy_report_schema_version": report_schema_version,
            "slo_policy_report_status": report_status,
            "slo_policy_report_final_decision": report_final_decision,
            "slo_policy_report_reason_key": report_reason_key,
            "slo_policy_report_reason_codes": report_reason_codes,
            "slo_policy_report_mtime_utc": report_mtime_utc,
            "slo_policy_report_age_seconds": report_age_seconds,
        },
        "contracts": {
            "slo_policy_report_schema_version_required": SLO_POLICY_SOURCE_SCHEMA_VERSION,
            "slo_policy_report_status_required": "pass",
            "slo_policy_report_final_decision_required": GO_DECISION,
            "slo_policy_report_reason_key_required": SLO_POLICY_SOURCE_REASON_KEY,
            "slo_policy_report_reason_codes_required": [],
            "slo_policy_max_age_seconds_required": max_age_seconds,
        },
    }


def _optional_incident_readiness_gate_inputs(
    args: argparse.Namespace,
) -> tuple[Path, int] | None:
    raw_report_file = getattr(args, "incident_readiness_report_file", "")
    raw_max_age = getattr(args, "incident_readiness_max_age_seconds", "")

    report_file = raw_report_file.strip() if isinstance(raw_report_file, str) else ""
    max_age_value = raw_max_age.strip() if isinstance(raw_max_age, str) else ""

    if not report_file and not max_age_value:
        return None
    if not report_file:
        fail(
            "--incident-readiness-report-file is required when "
            "--incident-readiness-max-age-seconds is provided"
        )

    if not max_age_value:
        max_age_seconds = DEFAULT_INCIDENT_READINESS_MAX_AGE_SECONDS
    else:
        max_age_seconds = parse_int("incident-readiness-max-age-seconds", max_age_value)
        if max_age_seconds < 1:
            fail("incident-readiness-max-age-seconds must be >= 1")

    return Path(report_file).resolve(), max_age_seconds


def _build_incident_readiness_gate(
    report_path: Path, max_age_seconds: int, reference_time: datetime
) -> dict[str, Any]:
    reason_codes: list[str] = []
    report_payload: dict[str, Any] | None = None
    report_schema_version = ""
    report_final_decision = ""
    report_evidence_output_contract_version = ""
    report_reason_taxonomy_schema_version = ""
    report_normalized_evidence_schema_version = ""
    report_staged_signoff_schema_version = ""
    report_staged_signoff_lineage_status = ""
    report_reason_codes: list[str] = []
    report_reason_codes_csv = ""
    report_mtime_utc = ""
    report_age_seconds = -1

    if max_age_seconds < 1:
        reason_codes.append("gonogo_incident_readiness_max_age_invalid")

    if not report_path.is_file():
        reason_codes.append("gonogo_incident_readiness_file_missing")
    else:
        mtime_utc = datetime.fromtimestamp(report_path.stat().st_mtime, tz=timezone.utc)
        report_mtime_utc = mtime_utc.strftime("%Y-%m-%dT%H:%M:%SZ")
        report_age_seconds = max(
            0, int((reference_time - mtime_utc).total_seconds())
        )
        if max_age_seconds >= 1 and report_age_seconds > max_age_seconds:
            reason_codes.append("gonogo_incident_readiness_freshness_window_exceeded")

        try:
            payload = load_json(report_path)
        except ContractError:
            reason_codes.append("gonogo_incident_readiness_invalid_json")
        else:
            if not isinstance(payload, dict):
                reason_codes.append("gonogo_incident_readiness_invalid_json")
            else:
                report_payload = payload

    if report_payload is not None:
        report_schema_version = str(report_payload.get("schema_version", ""))
        report_final_decision = str(report_payload.get("final_decision", ""))
        report_evidence_output_contract_version = str(
            report_payload.get("evidence_output_contract_version", "")
        )

        reason_taxonomy = report_payload.get("reason_taxonomy")
        if isinstance(reason_taxonomy, dict):
            report_reason_taxonomy_schema_version = str(
                reason_taxonomy.get("schema_version", "")
            )
        else:
            reason_codes.append(
                "gonogo_incident_readiness_reason_taxonomy_schema_mismatch"
            )

        normalized_evidence = report_payload.get("normalized_evidence")
        if isinstance(normalized_evidence, dict):
            report_normalized_evidence_schema_version = str(
                normalized_evidence.get("schema_version", "")
            )
        else:
            reason_codes.append(
                "gonogo_incident_readiness_normalized_evidence_schema_mismatch"
            )

        staged_signoff = report_payload.get("staged_rehearsal_signoff")
        if isinstance(staged_signoff, dict):
            report_staged_signoff_schema_version = str(
                staged_signoff.get("schema_version", "")
            )
            report_staged_signoff_lineage_status = str(
                staged_signoff.get("lineage_status", "")
            )
        else:
            reason_codes.append("gonogo_incident_readiness_staged_signoff_schema_mismatch")
            reason_codes.append("gonogo_incident_readiness_staged_signoff_status_not_verified")

        raw_reason_codes = report_payload.get("decision_reasons")
        if isinstance(raw_reason_codes, list):
            report_reason_codes = [
                value for value in raw_reason_codes if isinstance(value, str) and value
            ]
            report_reason_codes_csv = ",".join(report_reason_codes)
        else:
            reason_codes.append("gonogo_incident_readiness_reason_codes_unexpected")

        if report_schema_version != INCIDENT_READINESS_SOURCE_SCHEMA_VERSION:
            reason_codes.append("gonogo_incident_readiness_schema_mismatch")
        if report_final_decision != GO_DECISION:
            reason_codes.append("gonogo_incident_readiness_final_decision_not_go")
        if (
            report_evidence_output_contract_version
            != INCIDENT_READINESS_SOURCE_OUTPUT_CONTRACT_VERSION
        ):
            reason_codes.append(
                "gonogo_incident_readiness_output_contract_version_mismatch"
            )
        if (
            report_reason_taxonomy_schema_version
            != INCIDENT_READINESS_SOURCE_REASON_TAXONOMY_SCHEMA_VERSION
        ):
            reason_codes.append(
                "gonogo_incident_readiness_reason_taxonomy_schema_mismatch"
            )
        if (
            report_normalized_evidence_schema_version
            != INCIDENT_READINESS_SOURCE_NORMALIZED_EVIDENCE_SCHEMA_VERSION
        ):
            reason_codes.append(
                "gonogo_incident_readiness_normalized_evidence_schema_mismatch"
            )
        if (
            report_staged_signoff_schema_version
            != INCIDENT_READINESS_SOURCE_STAGED_SIGNOFF_SCHEMA_VERSION
        ):
            reason_codes.append(
                "gonogo_incident_readiness_staged_signoff_schema_mismatch"
            )
        if report_staged_signoff_lineage_status != "verified":
            reason_codes.append("gonogo_incident_readiness_staged_signoff_status_not_verified")
        if report_reason_codes != INCIDENT_READINESS_SOURCE_REASON_CODES:
            reason_codes.append("gonogo_incident_readiness_reason_codes_unexpected")

    reason_codes = sorted(set(reason_codes))
    final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION
    gate_status = "verified" if final_decision == GO_DECISION else "fail-closed"
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

    return {
        "schema_version": INCIDENT_READINESS_GATE_SCHEMA_VERSION,
        "reason_taxonomy_version": INCIDENT_READINESS_GATE_REASON_TAXONOMY_VERSION,
        "final_decision": final_decision,
        "status": gate_status,
        "reason_codes": reason_codes,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes_value": reason_codes_csv,
        "artifacts": {
            "incident_readiness_report_file": str(report_path),
            "incident_readiness_report_sha256": _artifact_sha256(report_path),
        },
        "observed": {
            "incident_readiness_report_schema_version": report_schema_version,
            "incident_readiness_report_final_decision": report_final_decision,
            "incident_readiness_report_evidence_output_contract_version": (
                report_evidence_output_contract_version
            ),
            "incident_readiness_report_reason_taxonomy_schema_version": (
                report_reason_taxonomy_schema_version
            ),
            "incident_readiness_report_normalized_evidence_schema_version": (
                report_normalized_evidence_schema_version
            ),
            "incident_readiness_report_staged_signoff_schema_version": (
                report_staged_signoff_schema_version
            ),
            "incident_readiness_report_staged_signoff_lineage_status": (
                report_staged_signoff_lineage_status
            ),
            "incident_readiness_report_reason_codes": report_reason_codes,
            "incident_readiness_report_reason_codes_csv": report_reason_codes_csv,
            "incident_readiness_report_mtime_utc": report_mtime_utc,
            "incident_readiness_report_age_seconds": report_age_seconds,
        },
        "contracts": {
            "incident_readiness_report_schema_version_required": (
                INCIDENT_READINESS_SOURCE_SCHEMA_VERSION
            ),
            "incident_readiness_report_final_decision_required": GO_DECISION,
            "incident_readiness_report_evidence_output_contract_version_required": (
                INCIDENT_READINESS_SOURCE_OUTPUT_CONTRACT_VERSION
            ),
            "incident_readiness_report_reason_taxonomy_schema_version_required": (
                INCIDENT_READINESS_SOURCE_REASON_TAXONOMY_SCHEMA_VERSION
            ),
            "incident_readiness_report_normalized_evidence_schema_version_required": (
                INCIDENT_READINESS_SOURCE_NORMALIZED_EVIDENCE_SCHEMA_VERSION
            ),
            "incident_readiness_report_staged_signoff_schema_version_required": (
                INCIDENT_READINESS_SOURCE_STAGED_SIGNOFF_SCHEMA_VERSION
            ),
            "incident_readiness_report_staged_signoff_lineage_status_required": (
                "verified"
            ),
            "incident_readiness_report_reason_codes_required": (
                list(INCIDENT_READINESS_SOURCE_REASON_CODES)
            ),
            "incident_readiness_max_age_seconds_required": max_age_seconds,
        },
    }


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
    combined_reason_taxonomy_version = ""
    combined_transport_reason_codes: list[str] = []
    combined_kolme_runtime_reason_code = ""
    gate_kolme_runtime_commit_failure_taxonomy_version = ""
    gate_kolme_fixture_profile = ""
    gate_kolme_fixture_profile_version = ""
    gate_kolme_fixture_profile_status = ""
    gate_combined_lane_marker_contract_status = ""
    if gate_report is not None:
        if gate_report.get("schema_version") != GO_NO_GO_GATE_SCHEMA:
            reason_codes.append("milestone_review_go_no_go_gate_schema_mismatch")
        gate_status = str(gate_report.get("status", ""))
        if gate_status != "pass":
            reason_codes.append("milestone_review_go_no_go_gate_status_mismatch")
        gate_final_decision = str(gate_report.get("final_decision", ""))
        if gate_final_decision != GO_DECISION:
            reason_codes.append("milestone_review_go_no_go_gate_final_decision_mismatch")

        combined_reason_taxonomy_version = str(gate_report.get("combined_reason_taxonomy_version", ""))
        if combined_reason_taxonomy_version != COMBINED_REASON_TAXONOMY_VERSION:
            reason_codes.append(
                "milestone_review_go_no_go_gate_combined_reason_taxonomy_version_mismatch"
            )

        combined_transport_reason_codes_raw = gate_report.get("combined_transport_reason_codes")
        if isinstance(combined_transport_reason_codes_raw, list):
            combined_transport_reason_codes = [
                value
                for value in combined_transport_reason_codes_raw
                if isinstance(value, str) and value
            ]
        if combined_transport_reason_codes != COMBINED_TRANSPORT_REASON_CODES:
            reason_codes.append(
                "milestone_review_go_no_go_gate_combined_transport_reason_codes_mismatch"
            )

        combined_kolme_runtime_reason_code = str(
            gate_report.get("combined_kolme_runtime_reason_code", "")
        )
        if combined_kolme_runtime_reason_code not in ALLOWED_COMBINED_KOLME_REASON_CODES:
            reason_codes.append(
                "milestone_review_go_no_go_gate_combined_kolme_runtime_reason_code_mismatch"
            )

        gate_kolme_runtime_commit_failure_taxonomy_version = str(
            gate_report.get("kolme_runtime_commit_failure_taxonomy_version", "")
        )
        if (
            gate_kolme_runtime_commit_failure_taxonomy_version
            != KOLME_RUNTIME_COMMIT_FAILURE_TAXONOMY_VERSION
        ):
            reason_codes.append(
                "milestone_review_go_no_go_gate_kolme_runtime_commit_failure_"
                "taxonomy_version_mismatch"
            )

        gate_kolme_fixture_profile = str(gate_report.get("kolme_fixture_profile", ""))
        if gate_kolme_fixture_profile != KOLME_RUNTIME_COMMIT_PROFILE:
            reason_codes.append("milestone_review_go_no_go_gate_kolme_fixture_profile_mismatch")

        gate_kolme_fixture_profile_version = str(gate_report.get("kolme_fixture_profile_version", ""))
        if gate_kolme_fixture_profile_version != KOLME_RUNTIME_COMMIT_PROFILE_VERSION:
            reason_codes.append(
                "milestone_review_go_no_go_gate_kolme_fixture_profile_version_mismatch"
            )

        gate_kolme_fixture_profile_status = str(gate_report.get("kolme_fixture_profile_status", ""))
        if gate_kolme_fixture_profile_status not in {"planned", "verified"}:
            reason_codes.append(
                "milestone_review_go_no_go_gate_kolme_fixture_profile_status_mismatch"
            )

        gate_combined_lane_marker_contract_status = str(
            gate_report.get("combined_lane_marker_contract_status", "")
        )
        if gate_combined_lane_marker_contract_status != "verified":
            reason_codes.append(
                "milestone_review_go_no_go_gate_combined_lane_marker_contract_status_mismatch"
            )

    reason_codes = sorted(set(reason_codes))
    final_decision = GO_DECISION if not reason_codes else NO_GO_DECISION
    lineage_status = "verified" if final_decision == GO_DECISION else "fail-closed"
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)

    artifacts: dict[str, str] = {}
    for field_name, path in artifact_paths.items():
        artifacts[field_name] = str(path)
        artifacts[f"{field_name[:-5]}sha256"] = _artifact_sha256(path)
    artifacts["operator_runbook_doc_file"] = str(operator_runbook_doc)
    artifacts["operator_runbook_doc_sha256"] = _artifact_sha256(operator_runbook_doc)

    return {
        "schema_version": MILESTONE_REVIEW_SCHEMA_VERSION,
        "reason_taxonomy_version": LIVE_GONOGO_REASON_TAXONOMY_VERSION,
        "final_decision": final_decision,
        "lineage_status": lineage_status,
        "reason_codes": reason_codes,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes_value": reason_codes_csv,
        "artifacts": artifacts,
        "observed": {
            "deployment_preflight_summary_status": preflight_status,
            "deployment_preflight_policy_final_decision": preflight_policy_final_decision,
            "live_node_validation_summary_status": live_status,
            "live_node_validation_policy_final_decision": live_policy_final_decision,
            "go_no_go_gate_status": gate_status,
            "go_no_go_gate_final_decision": gate_final_decision,
            "go_no_go_gate_combined_reason_taxonomy_version": combined_reason_taxonomy_version,
            "go_no_go_gate_combined_transport_reason_codes": combined_transport_reason_codes,
            "go_no_go_gate_combined_kolme_runtime_reason_code": combined_kolme_runtime_reason_code,
            "go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version": (
                gate_kolme_runtime_commit_failure_taxonomy_version
            ),
            "go_no_go_gate_kolme_fixture_profile": gate_kolme_fixture_profile,
            "go_no_go_gate_kolme_fixture_profile_version": gate_kolme_fixture_profile_version,
            "go_no_go_gate_kolme_fixture_profile_status": gate_kolme_fixture_profile_status,
            "go_no_go_gate_combined_lane_marker_contract_status": (
                gate_combined_lane_marker_contract_status
            ),
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
            "go_no_go_gate_combined_reason_taxonomy_version_required": (
                COMBINED_REASON_TAXONOMY_VERSION
            ),
            "go_no_go_gate_combined_transport_reason_codes_required": list(
                COMBINED_TRANSPORT_REASON_CODES
            ),
            "go_no_go_gate_combined_kolme_runtime_reason_codes_allowed": sorted(
                ALLOWED_COMBINED_KOLME_REASON_CODES
            ),
            "go_no_go_gate_kolme_runtime_commit_failure_taxonomy_version_required": (
                KOLME_RUNTIME_COMMIT_FAILURE_TAXONOMY_VERSION
            ),
            "go_no_go_gate_kolme_fixture_profile_required": KOLME_RUNTIME_COMMIT_PROFILE,
            "go_no_go_gate_kolme_fixture_profile_version_required": (
                KOLME_RUNTIME_COMMIT_PROFILE_VERSION
            ),
            "go_no_go_gate_kolme_fixture_profile_status_allowed": ["planned", "verified"],
            "go_no_go_gate_combined_lane_marker_contract_status_required": "verified",
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
            "reason_taxonomy_version",
            "final_decision",
            "lineage_status",
            "reason_codes",
            "reason_codes_csv",
            "reason_codes_value",
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


def _validated_expected_tls_evidence_gate(
    payload: dict[str, object], reference_time: datetime
) -> tuple[dict[str, Any], str]:
    tls_evidence_gate = payload.get("tls_evidence_gate")
    if not isinstance(tls_evidence_gate, dict):
        fail("bundle field 'tls_evidence_gate' must be an object when provided")

    require_keys(
        tls_evidence_gate,
        (
            "schema_version",
            "reason_taxonomy_version",
            "final_decision",
            "status",
            "reason_codes",
            "reason_codes_csv",
            "reason_codes_value",
            "artifacts",
            "observed",
            "contracts",
        ),
    )

    artifacts = tls_evidence_gate.get("artifacts")
    if not isinstance(artifacts, dict):
        fail("tls_evidence_gate.artifacts must be an object")
    report_file = artifacts.get("tls_evidence_report_file")
    if not isinstance(report_file, str) or not report_file.strip():
        fail("tls_evidence_gate.artifacts.tls_evidence_report_file must be a non-empty string")

    contracts = tls_evidence_gate.get("contracts")
    if not isinstance(contracts, dict):
        fail("tls_evidence_gate.contracts must be an object")
    max_age_seconds = contracts.get("tls_evidence_max_age_seconds_required")
    if not isinstance(max_age_seconds, int):
        fail("tls_evidence_gate.contracts.tls_evidence_max_age_seconds_required must be an integer")

    expected_gate = _build_tls_evidence_gate(
        Path(report_file).resolve(), max_age_seconds, reference_time
    )
    if tls_evidence_gate != expected_gate:
        fail(
            "tls evidence gate convergence mismatch: "
            "expected deterministic tls evidence completeness/freshness markers and decision surface"
        )

    expected_decision = expected_gate["final_decision"]
    if not isinstance(expected_decision, str):
        fail("tls evidence gate final decision must be a string")
    return expected_gate, expected_decision


def _validated_expected_audit_integrity_gate(
    payload: dict[str, object], reference_time: datetime
) -> tuple[dict[str, Any], str]:
    audit_integrity_gate = payload.get("audit_integrity_gate")
    if not isinstance(audit_integrity_gate, dict):
        fail("bundle field 'audit_integrity_gate' must be an object when provided")

    require_keys(
        audit_integrity_gate,
        (
            "schema_version",
            "reason_taxonomy_version",
            "final_decision",
            "status",
            "reason_codes",
            "reason_codes_csv",
            "reason_codes_value",
            "artifacts",
            "observed",
            "contracts",
        ),
    )

    artifacts = audit_integrity_gate.get("artifacts")
    if not isinstance(artifacts, dict):
        fail("audit_integrity_gate.artifacts must be an object")
    report_file = artifacts.get("audit_integrity_report_file")
    if not isinstance(report_file, str) or not report_file.strip():
        fail(
            "audit_integrity_gate.artifacts.audit_integrity_report_file must be a non-empty string"
        )

    contracts = audit_integrity_gate.get("contracts")
    if not isinstance(contracts, dict):
        fail("audit_integrity_gate.contracts must be an object")
    max_age_seconds = contracts.get("audit_integrity_max_age_seconds_required")
    if not isinstance(max_age_seconds, int):
        fail(
            "audit_integrity_gate.contracts.audit_integrity_max_age_seconds_required must be an integer"
        )

    expected_gate = _build_audit_integrity_gate(
        Path(report_file).resolve(), max_age_seconds, reference_time
    )
    if audit_integrity_gate != expected_gate:
        fail(
            "audit integrity gate convergence mismatch: "
            "expected deterministic audit integrity markers and decision surface"
        )

    expected_decision = expected_gate["final_decision"]
    if not isinstance(expected_decision, str):
        fail("audit integrity gate final decision must be a string")
    return expected_gate, expected_decision


def _validated_expected_slo_policy_gate(
    payload: dict[str, object], reference_time: datetime
) -> tuple[dict[str, Any], str]:
    slo_policy_gate = payload.get("slo_policy_gate")
    if not isinstance(slo_policy_gate, dict):
        fail("bundle field 'slo_policy_gate' must be an object when provided")

    require_keys(
        slo_policy_gate,
        (
            "schema_version",
            "reason_taxonomy_version",
            "final_decision",
            "status",
            "reason_codes",
            "reason_codes_csv",
            "reason_codes_value",
            "artifacts",
            "observed",
            "contracts",
        ),
    )

    artifacts = slo_policy_gate.get("artifacts")
    if not isinstance(artifacts, dict):
        fail("slo_policy_gate.artifacts must be an object")
    report_file = artifacts.get("slo_policy_report_file")
    if not isinstance(report_file, str) or not report_file.strip():
        fail("slo_policy_gate.artifacts.slo_policy_report_file must be a non-empty string")

    contracts = slo_policy_gate.get("contracts")
    if not isinstance(contracts, dict):
        fail("slo_policy_gate.contracts must be an object")
    max_age_seconds = contracts.get("slo_policy_max_age_seconds_required")
    if not isinstance(max_age_seconds, int):
        fail(
            "slo_policy_gate.contracts.slo_policy_max_age_seconds_required must be an integer"
        )

    expected_gate = _build_slo_policy_gate(
        Path(report_file).resolve(), max_age_seconds, reference_time
    )
    if slo_policy_gate != expected_gate:
        fail(
            "slo policy gate convergence mismatch: "
            "expected deterministic slo threshold markers and decision surface"
        )

    expected_decision = expected_gate["final_decision"]
    if not isinstance(expected_decision, str):
        fail("slo policy gate final decision must be a string")
    return expected_gate, expected_decision


def _validated_expected_incident_readiness_gate(
    payload: dict[str, object], reference_time: datetime
) -> tuple[dict[str, Any], str]:
    incident_readiness_gate = payload.get("incident_readiness_gate")
    if not isinstance(incident_readiness_gate, dict):
        fail("bundle field 'incident_readiness_gate' must be an object when provided")

    require_keys(
        incident_readiness_gate,
        (
            "schema_version",
            "reason_taxonomy_version",
            "final_decision",
            "status",
            "reason_codes",
            "reason_codes_csv",
            "reason_codes_value",
            "artifacts",
            "observed",
            "contracts",
        ),
    )

    artifacts = incident_readiness_gate.get("artifacts")
    if not isinstance(artifacts, dict):
        fail("incident_readiness_gate.artifacts must be an object")
    report_file = artifacts.get("incident_readiness_report_file")
    if not isinstance(report_file, str) or not report_file.strip():
        fail(
            "incident_readiness_gate.artifacts.incident_readiness_report_file must be a non-empty string"
        )

    contracts = incident_readiness_gate.get("contracts")
    if not isinstance(contracts, dict):
        fail("incident_readiness_gate.contracts must be an object")
    max_age_seconds = contracts.get("incident_readiness_max_age_seconds_required")
    if not isinstance(max_age_seconds, int):
        fail(
            "incident_readiness_gate.contracts.incident_readiness_max_age_seconds_required must be an integer"
        )

    expected_gate = _build_incident_readiness_gate(
        Path(report_file).resolve(), max_age_seconds, reference_time
    )
    if incident_readiness_gate != expected_gate:
        fail(
            "incident readiness gate convergence mismatch: "
            "expected deterministic readiness bundle schema markers and decision surface"
        )

    expected_decision = expected_gate["final_decision"]
    if not isinstance(expected_decision, str):
        fail("incident readiness gate final decision must be a string")
    return expected_gate, expected_decision


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

    generated_at_dt = datetime.now(timezone.utc).replace(microsecond=0)
    generated_at = generated_at_dt.strftime("%Y-%m-%dT%H:%M:%SZ")

    payload: dict[str, Any] = {
        "schema_version": SCHEMA_VERSION,
        "generated_at": generated_at,
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

    tls_evidence_gate = None
    tls_evidence_gate_inputs = _optional_tls_evidence_gate_inputs(args)
    if tls_evidence_gate_inputs is not None:
        tls_report_file, tls_max_age_seconds = tls_evidence_gate_inputs
        tls_evidence_gate = _build_tls_evidence_gate(
            tls_report_file, tls_max_age_seconds, generated_at_dt
        )
        payload["tls_evidence_gate"] = tls_evidence_gate
        expected_go = expected_go and tls_evidence_gate["final_decision"] == GO_DECISION

    audit_integrity_gate = None
    audit_integrity_gate_inputs = _optional_audit_integrity_gate_inputs(args)
    if audit_integrity_gate_inputs is not None:
        audit_report_file, audit_max_age_seconds = audit_integrity_gate_inputs
        audit_integrity_gate = _build_audit_integrity_gate(
            audit_report_file, audit_max_age_seconds, generated_at_dt
        )
        payload["audit_integrity_gate"] = audit_integrity_gate
        expected_go = expected_go and audit_integrity_gate["final_decision"] == GO_DECISION

    slo_policy_gate = None
    slo_policy_gate_inputs = _optional_slo_policy_gate_inputs(args)
    if slo_policy_gate_inputs is not None:
        slo_policy_report_file, slo_policy_max_age_seconds = slo_policy_gate_inputs
        slo_policy_gate = _build_slo_policy_gate(
            slo_policy_report_file, slo_policy_max_age_seconds, generated_at_dt
        )
        payload["slo_policy_gate"] = slo_policy_gate
        expected_go = expected_go and slo_policy_gate["final_decision"] == GO_DECISION

    incident_readiness_gate = None
    incident_readiness_gate_inputs = _optional_incident_readiness_gate_inputs(args)
    if incident_readiness_gate_inputs is not None:
        (
            incident_readiness_report_file,
            incident_readiness_max_age_seconds,
        ) = incident_readiness_gate_inputs
        incident_readiness_gate = _build_incident_readiness_gate(
            incident_readiness_report_file,
            incident_readiness_max_age_seconds,
            generated_at_dt,
        )
        payload["incident_readiness_gate"] = incident_readiness_gate
        expected_go = (
            expected_go and incident_readiness_gate["final_decision"] == GO_DECISION
        )

    final_decision = GO_DECISION if expected_go else NO_GO_DECISION
    payload["final_decision"] = final_decision

    output_path = Path(args.output_file)
    write_json(output_path, payload)

    print("status=generated")
    print(f"bundle_file={output_path}")
    if milestone_review_bundle is not None:
        print(f"milestone_review_final_decision={milestone_review_bundle['final_decision']}")
        print(f"live_gonogo_reason_taxonomy_version={LIVE_GONOGO_REASON_TAXONOMY_VERSION}")
        print(f"live_gonogo_reason_codes_csv={milestone_review_bundle['reason_codes_csv']}")
    if tls_evidence_gate is not None:
        print(f"tls_evidence_gate_final_decision={tls_evidence_gate['final_decision']}")
        print(f"tls_evidence_reason_taxonomy_version={TLS_EVIDENCE_GATE_REASON_TAXONOMY_VERSION}")
        print(f"tls_evidence_reason_codes_csv={tls_evidence_gate['reason_codes_csv']}")
    if audit_integrity_gate is not None:
        print(
            "audit_integrity_gate_final_decision="
            f"{audit_integrity_gate['final_decision']}"
        )
        print(
            "audit_integrity_reason_taxonomy_version="
            f"{AUDIT_INTEGRITY_GATE_REASON_TAXONOMY_VERSION}"
        )
        print(
            "audit_integrity_reason_codes_csv="
            f"{audit_integrity_gate['reason_codes_csv']}"
        )
    if slo_policy_gate is not None:
        print(f"slo_policy_gate_final_decision={slo_policy_gate['final_decision']}")
        print(
            "slo_policy_reason_taxonomy_version="
            f"{SLO_POLICY_GATE_REASON_TAXONOMY_VERSION}"
        )
        print(f"slo_policy_reason_codes_csv={slo_policy_gate['reason_codes_csv']}")
    if incident_readiness_gate is not None:
        print(
            "incident_readiness_gate_final_decision="
            f"{incident_readiness_gate['final_decision']}"
        )
        print(
            "incident_readiness_reason_taxonomy_version="
            f"{INCIDENT_READINESS_GATE_REASON_TAXONOMY_VERSION}"
        )
        print(
            "incident_readiness_reason_codes_csv="
            f"{incident_readiness_gate['reason_codes_csv']}"
        )
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

    generated_at = _parse_utc_timestamp(payload.get("generated_at"), "generated_at")

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

    milestone_bundle: dict[str, Any] | None = None
    milestone_decision = GO_DECISION
    if "milestone_review_bundle" in payload:
        milestone_bundle, milestone_decision = _validated_expected_milestone_bundle(payload)
        expected_go = expected_go and milestone_decision == GO_DECISION

    tls_evidence_gate_decision = GO_DECISION
    if "tls_evidence_gate" in payload:
        _, tls_evidence_gate_decision = _validated_expected_tls_evidence_gate(
            payload, generated_at
        )
        expected_go = expected_go and tls_evidence_gate_decision == GO_DECISION

    audit_integrity_gate_decision = GO_DECISION
    if "audit_integrity_gate" in payload:
        _, audit_integrity_gate_decision = _validated_expected_audit_integrity_gate(
            payload, generated_at
        )
        expected_go = expected_go and audit_integrity_gate_decision == GO_DECISION

    slo_policy_gate_decision = GO_DECISION
    if "slo_policy_gate" in payload:
        _, slo_policy_gate_decision = _validated_expected_slo_policy_gate(
            payload, generated_at
        )
        expected_go = expected_go and slo_policy_gate_decision == GO_DECISION

    incident_readiness_gate_decision = GO_DECISION
    if "incident_readiness_gate" in payload:
        _, incident_readiness_gate_decision = _validated_expected_incident_readiness_gate(
            payload, generated_at
        )
        expected_go = expected_go and incident_readiness_gate_decision == GO_DECISION

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
        print(f"live_gonogo_reason_taxonomy_version={LIVE_GONOGO_REASON_TAXONOMY_VERSION}")
        if milestone_bundle is not None:
            print(f"live_gonogo_reason_codes_csv={milestone_bundle['reason_codes_csv']}")
    if "tls_evidence_gate" in payload:
        print(f"tls_evidence_gate_final_decision={tls_evidence_gate_decision}")
    if "audit_integrity_gate" in payload:
        print(f"audit_integrity_gate_final_decision={audit_integrity_gate_decision}")
    if "slo_policy_gate" in payload:
        print(f"slo_policy_gate_final_decision={slo_policy_gate_decision}")
    if "incident_readiness_gate" in payload:
        print(
            "incident_readiness_gate_final_decision="
            f"{incident_readiness_gate_decision}"
        )
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
    generate.add_argument("--tls-evidence-report-file", default="")
    generate.add_argument("--tls-evidence-max-age-seconds", default="")
    generate.add_argument("--audit-integrity-report-file", default="")
    generate.add_argument("--audit-integrity-max-age-seconds", default="")
    generate.add_argument("--slo-policy-report-file", default="")
    generate.add_argument("--slo-policy-max-age-seconds", default="")
    generate.add_argument("--incident-readiness-report-file", default="")
    generate.add_argument("--incident-readiness-max-age-seconds", default="")
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
