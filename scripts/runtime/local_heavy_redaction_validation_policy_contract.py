#!/usr/bin/env python3
"""Policy checker for local-heavy redaction validation lane artifacts."""

from __future__ import annotations

import argparse
import json
from pathlib import Path

RUNNER_REPORT_SCHEMA = "kamn.runtime.local-heavy-redaction-validation-lane-report.v1"
RUNNER_ARTIFACT_SCHEMA = "kamn.runtime.local-heavy-redaction-validation-artifact-schema.v1"
RUNNER_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.local-heavy-redaction-validation-reason-taxonomy.v1"
)
RUNNER_REASON_CODES_CSV = (
    "local_heavy_redaction_sensitive_pattern_detected,"
    "local_heavy_redaction_runtime_budget_exceeded"
)

POLICY_REPORT_SCHEMA = "kamn.runtime.local-heavy-redaction-validation-policy-report.v1"
POLICY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.local-heavy-redaction-validation-policy-reason-taxonomy.v1"
)
POLICY_REASON_CODES_CSV = (
    "redaction_policy_required_field_missing,"
    "redaction_policy_marker_mismatch,"
    "redaction_policy_reason_taxonomy_mismatch,"
    "redaction_policy_profile_contract_mismatch,"
    "redaction_policy_docs_marker_parity_mismatch,"
    "ci_fast_gate_failed,"
    "redaction_policy_expected_decision_mismatch,"
    "redaction_policy_violation"
)
POLICY_REASON_CODES_ORDER = POLICY_REASON_CODES_CSV.split(",")

DEFAULT_STRATEGY_DOC = Path("docs/ci/strategy.md")
DEFAULT_OPS_DOC = Path("docs/ops/configuration.md")

REQUIRED_REPORT_FIELDS = (
    "schema_version",
    "artifact_schema_version",
    "reason_taxonomy_version",
    "reason_codes_csv",
    "status",
    "final_decision",
    "lane_mode",
    "profile",
    "profile_status",
    "reason_code",
    "leak_marker_status",
    "leak_detection_count",
    "leaked_pattern_ids_csv",
    "ci_fast_gate",
    "run_mode_command_status",
    "command_count",
    "performance_budget_status",
    "elapsed_seconds",
    "max_seconds",
)

STRATEGY_REQUIRED_MARKERS = (
    "## Local-Heavy Redaction Validation Policy Checker Contract",
    "bash scripts/runtime/check_local_heavy_redaction_validation_policy.sh --report-file "
    "/tmp/local-heavy-redaction-validation-baseline.json --expected-final-decision GO "
    "--ci-fast-gate PASS --strategy-doc docs/ci/strategy.md --ops-doc "
    "docs/ops/configuration.md --output-json "
    "/tmp/local-heavy-redaction-validation-policy-report.json",
    "bash scripts/runtime/test_check_local_heavy_redaction_validation_policy.sh",
    "local_heavy_redaction_validation_policy_reason_taxonomy_version="
    "kamn.runtime.local-heavy-redaction-validation-policy-reason-taxonomy.v1",
    "local_heavy_redaction_validation_policy_reason_codes_csv="
    "redaction_policy_required_field_missing,redaction_policy_marker_mismatch,"
    "redaction_policy_reason_taxonomy_mismatch,redaction_policy_profile_contract_mismatch,"
    "redaction_policy_docs_marker_parity_mismatch,ci_fast_gate_failed,"
    "redaction_policy_expected_decision_mismatch,redaction_policy_violation",
    "local_heavy_redaction_validation_policy_strategy_doc_path=docs/ci/strategy.md",
    "local_heavy_redaction_validation_policy_ops_doc_path=docs/ops/configuration.md",
    "local_heavy_redaction_validation_policy_runner_report_schema_version="
    "kamn.runtime.local-heavy-redaction-validation-lane-report.v1",
    "local_heavy_redaction_validation_policy_runner_reason_taxonomy_version="
    "kamn.runtime.local-heavy-redaction-validation-reason-taxonomy.v1",
    "local_heavy_redaction_validation_policy_runner_reason_codes_csv="
    "local_heavy_redaction_sensitive_pattern_detected,"
    "local_heavy_redaction_runtime_budget_exceeded",
)

OPS_REQUIRED_MARKERS = (
    "## Local-Heavy Redaction Validation Lane Artifact Contract (Issue #4079)",
    "local_heavy_redaction_validation_lane_schema_version="
    "kamn.runtime.local-heavy-redaction-validation-lane-report.v1",
    "local_heavy_redaction_validation_artifact_schema_version="
    "kamn.runtime.local-heavy-redaction-validation-artifact-schema.v1",
    "local_heavy_redaction_validation_reason_taxonomy_version="
    "kamn.runtime.local-heavy-redaction-validation-reason-taxonomy.v1",
    "local_heavy_redaction_validation_reason_codes_csv="
    "local_heavy_redaction_sensitive_pattern_detected,"
    "local_heavy_redaction_runtime_budget_exceeded",
    "local_heavy_redaction_validation_required_profiles_csv=baseline,injected-leak",
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--report-file", required=True, type=Path)
    parser.add_argument(
        "--expected-final-decision",
        required=True,
        choices=("GO", "NO-GO"),
    )
    parser.add_argument("--ci-fast-gate", required=True, choices=("PASS", "FAIL"))
    parser.add_argument("--strategy-doc", type=Path, default=DEFAULT_STRATEGY_DOC)
    parser.add_argument("--ops-doc", type=Path, default=DEFAULT_OPS_DOC)
    parser.add_argument("--output-json", type=Path)
    return parser.parse_args()


def load_json_dict(path: Path) -> dict[str, object]:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("report-json-must-be-object")
    return payload


def normalize_reason_codes(raw_reason_codes: list[str]) -> list[str]:
    observed = set(raw_reason_codes)
    normalized: list[str] = []
    for reason_code in POLICY_REASON_CODES_ORDER:
        if reason_code in observed:
            normalized.append(reason_code)
    return normalized


def reason_codes_value(reason_codes: list[str]) -> str:
    return "none" if not reason_codes else ",".join(reason_codes)


def profile_expected_markers(profile: str) -> tuple[str, str, str, int, str]:
    if profile == "baseline":
        return ("pass", "GO", "verified", 0, "clear")
    if profile == "injected-leak":
        return (
            "fail",
            "NO-GO",
            "failed",
            3,
            "detected",
        )
    return ("invalid", "INVALID", "invalid", -1, "invalid")


def evaluate_marker_parity(doc_path: Path, required_markers: tuple[str, ...]) -> str:
    try:
        text = doc_path.read_text(encoding="utf-8")
    except OSError:
        return "violation"

    missing_markers = [marker for marker in required_markers if marker not in text]
    return "violation" if missing_markers else "verified"


def main() -> int:
    args = parse_args()

    raw_reason_codes: list[str] = []
    marker_status = "verified"
    reason_taxonomy_status = "verified"
    profile_contract_status = "verified"

    if not args.report_file.exists():
        raw_reason_codes.append("redaction_policy_required_field_missing")
        payload: dict[str, object] = {}
    else:
        try:
            payload = load_json_dict(args.report_file)
        except Exception:
            payload = {}
            raw_reason_codes.append("redaction_policy_marker_mismatch")

    missing_fields = [
        field_name for field_name in REQUIRED_REPORT_FIELDS if field_name not in payload
    ]
    if missing_fields:
        raw_reason_codes.append("redaction_policy_required_field_missing")

    if payload.get("schema_version") != RUNNER_REPORT_SCHEMA:
        marker_status = "violation"
        raw_reason_codes.append("redaction_policy_marker_mismatch")
    if payload.get("artifact_schema_version") != RUNNER_ARTIFACT_SCHEMA:
        marker_status = "violation"
        raw_reason_codes.append("redaction_policy_marker_mismatch")

    if payload.get("reason_taxonomy_version") != RUNNER_REASON_TAXONOMY_VERSION:
        reason_taxonomy_status = "violation"
        raw_reason_codes.append("redaction_policy_reason_taxonomy_mismatch")
    if payload.get("reason_codes_csv") != RUNNER_REASON_CODES_CSV:
        reason_taxonomy_status = "violation"
        raw_reason_codes.append("redaction_policy_reason_taxonomy_mismatch")

    if payload.get("ci_fast_gate") != args.ci_fast_gate:
        raw_reason_codes.append("ci_fast_gate_failed")
    if args.ci_fast_gate != "PASS":
        raw_reason_codes.append("ci_fast_gate_failed")

    lane_mode = payload.get("lane_mode")
    if lane_mode not in ("dry-run", "run"):
        profile_contract_status = "violation"
        raw_reason_codes.append("redaction_policy_profile_contract_mismatch")

    profile = payload.get("profile")
    if profile not in ("baseline", "injected-leak"):
        profile_contract_status = "violation"
        raw_reason_codes.append("redaction_policy_profile_contract_mismatch")
    else:
        expected_status, expected_final_decision, expected_profile_status, expected_leak_count, expected_leak_marker = profile_expected_markers(
            profile
        )
        if payload.get("status") != expected_status:
            profile_contract_status = "violation"
            raw_reason_codes.append("redaction_policy_profile_contract_mismatch")
        if payload.get("final_decision") != expected_final_decision:
            profile_contract_status = "violation"
            raw_reason_codes.append("redaction_policy_profile_contract_mismatch")
        if payload.get("profile_status") != expected_profile_status:
            profile_contract_status = "violation"
            raw_reason_codes.append("redaction_policy_profile_contract_mismatch")
        if payload.get("leak_marker_status") != expected_leak_marker:
            profile_contract_status = "violation"
            raw_reason_codes.append("redaction_policy_profile_contract_mismatch")
        if payload.get("leak_detection_count") != expected_leak_count:
            profile_contract_status = "violation"
            raw_reason_codes.append("redaction_policy_profile_contract_mismatch")

        if profile == "baseline" and payload.get("reason_code") != "none":
            profile_contract_status = "violation"
            raw_reason_codes.append("redaction_policy_profile_contract_mismatch")
        if profile == "injected-leak" and payload.get("reason_code") != "local_heavy_redaction_sensitive_pattern_detected":
            profile_contract_status = "violation"
            raw_reason_codes.append("redaction_policy_profile_contract_mismatch")

    strategy_parity_status = evaluate_marker_parity(args.strategy_doc, STRATEGY_REQUIRED_MARKERS)
    ops_parity_status = evaluate_marker_parity(args.ops_doc, OPS_REQUIRED_MARKERS)
    docs_marker_parity_status = (
        "verified"
        if strategy_parity_status == "verified" and ops_parity_status == "verified"
        else "violation"
    )
    if docs_marker_parity_status != "verified":
        raw_reason_codes.append("redaction_policy_docs_marker_parity_mismatch")

    if payload.get("final_decision") != args.expected_final_decision:
        raw_reason_codes.append("redaction_policy_expected_decision_mismatch")

    normalized_reason_codes = normalize_reason_codes(raw_reason_codes)
    if raw_reason_codes and not normalized_reason_codes:
        normalized_reason_codes = ["redaction_policy_violation"]

    status = "pass" if not normalized_reason_codes else "fail"
    final_decision = "GO" if status == "pass" else "NO-GO"

    if final_decision != args.expected_final_decision:
        if "redaction_policy_expected_decision_mismatch" not in normalized_reason_codes:
            normalized_reason_codes.append("redaction_policy_expected_decision_mismatch")
        normalized_reason_codes = normalize_reason_codes(normalized_reason_codes)
        status = "pass" if not normalized_reason_codes else "fail"
        final_decision = "GO" if status == "pass" else "NO-GO"

    reason_value = reason_codes_value(normalized_reason_codes)
    policy_status = "verified" if status == "pass" else "violation"
    promotion_reason_code = "none" if status == "pass" else normalized_reason_codes[0]
    promotion_reason_mapping_status = (
        "verified"
        if (
            (status == "pass" and promotion_reason_code == "none")
            or (
                status == "fail"
                and promotion_reason_code in normalized_reason_codes
                and promotion_reason_code != "none"
            )
        )
        else "violation"
    )

    report_payload = {
        "schema_version": POLICY_REPORT_SCHEMA,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": POLICY_REASON_TAXONOMY_VERSION,
        "reason_codes_csv": POLICY_REASON_CODES_CSV,
        "reason_codes": normalized_reason_codes,
        "reason_codes_value": reason_value,
        "redaction_policy_status": policy_status,
        "redaction_policy_marker_status": marker_status,
        "redaction_policy_reason_taxonomy_status": reason_taxonomy_status,
        "redaction_policy_profile_contract_status": profile_contract_status,
        "redaction_policy_docs_marker_parity_status": docs_marker_parity_status,
        "redaction_policy_strategy_marker_parity_status": strategy_parity_status,
        "redaction_policy_ops_marker_parity_status": ops_parity_status,
        "promotion_decision_reason_mapping_status": promotion_reason_mapping_status,
        "promotion_decision_reason_code": promotion_reason_code,
        "inputs": {
            "report_file": str(args.report_file),
            "expected_final_decision": args.expected_final_decision,
            "ci_fast_gate": args.ci_fast_gate,
            "strategy_doc": str(args.strategy_doc),
            "ops_doc": str(args.ops_doc),
        },
    }

    if args.output_json:
        args.output_json.parent.mkdir(parents=True, exist_ok=True)
        args.output_json.write_text(
            json.dumps(report_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={POLICY_REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={POLICY_REASON_CODES_CSV}")
    print(f"reason_codes_value={reason_value}")
    print(f"redaction_policy_status={policy_status}")
    print(f"redaction_policy_marker_status={marker_status}")
    print(f"redaction_policy_reason_taxonomy_status={reason_taxonomy_status}")
    print(f"redaction_policy_profile_contract_status={profile_contract_status}")
    print(f"redaction_policy_docs_marker_parity_status={docs_marker_parity_status}")
    print(f"redaction_policy_strategy_marker_parity_status={strategy_parity_status}")
    print(f"redaction_policy_ops_marker_parity_status={ops_parity_status}")
    print(f"promotion_decision_reason_mapping_status={promotion_reason_mapping_status}")
    print(f"promotion_decision_reason_code={promotion_reason_code}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
