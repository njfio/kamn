#!/usr/bin/env python3
"""Fail-closed CI dry-run lifecycle tamper governance checker."""

from __future__ import annotations

import argparse
import json
import re
import time
from pathlib import Path

DEFAULT_THRESHOLD_FILE = Path("fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env")
DEFAULT_STRATEGY_DOC = Path("docs/ci/strategy.md")
DEFAULT_OPS_DOC = Path("docs/ops/configuration.md")
DEFAULT_WORKFLOW_FILE = Path(".github/workflows/ci-fast-gate.yml")
DEFAULT_CI_TOOLS_FILE = Path("scripts/ci/test_ci_tools.sh")

FALLBACK_REASON_CODES = [
    "lifecycle_ci_dry_run_argument_invalid",
    "lifecycle_ci_dry_run_threshold_contract_violation",
    "lifecycle_ci_dry_run_report_contract_violation",
    "lifecycle_ci_dry_run_lifecycle_marker_parity_drift",
    "lifecycle_ci_dry_run_go_no_go_marker_parity_drift",
    "lifecycle_ci_dry_run_runtime_budget_exceeded",
    "lifecycle_ci_dry_run_fast_mode_selector_drift",
    "lifecycle_ci_dry_run_workflow_exclusion_drift",
    "lifecycle_ci_dry_run_docs_marker_parity_drift",
    "lifecycle_ci_dry_run_docs_remediation_marker_missing",
]

REQUIRED_THRESHOLD_KEYS = {
    "LIFECYCLE_CI_DRY_RUN_POLICY_SCHEMA_VERSION",
    "LIFECYCLE_CI_DRY_RUN_REASON_TAXONOMY_VERSION",
    "LIFECYCLE_CI_DRY_RUN_REASON_CODES_CSV",
    "LIFECYCLE_CI_DRY_RUN_MAX_SECONDS",
    "LIFECYCLE_ARTIFACT_EXPECTED_SCHEMA_VERSION",
    "LIFECYCLE_ARTIFACT_EXPECTED_ARTIFACT_SCHEMA_VERSION",
    "LIFECYCLE_ARTIFACT_EXPECTED_REASON_TAXONOMY_VERSION",
    "LIFECYCLE_ARTIFACT_EXPECTED_REASON_CODES_CSV",
    "LIFECYCLE_ARTIFACT_EXPECTED_FINAL_DECISION",
    "LIFECYCLE_ARTIFACT_REQUIRED_HASH_FIELDS_CSV",
    "GO_NO_GO_EXPECTED_SCHEMA_VERSION",
    "GO_NO_GO_EXPECTED_REASON_TAXONOMY_VERSION",
    "GO_NO_GO_EXPECTED_STATUS",
    "GO_NO_GO_EXPECTED_FINAL_DECISION",
    "GO_NO_GO_EXPECTED_LANE_MODE",
    "GO_NO_GO_EXPECTED_FAST_GATE_ELIGIBLE",
    "GO_NO_GO_EXPECTED_FAST_GATE_SCOPE",
    "GO_NO_GO_EXPECTED_FAST_GATE_EXCLUSION_REASON_CODE",
    "GO_NO_GO_EXPECTED_RUN_MODE_COMMAND_STATUS",
    "GO_NO_GO_EXPECTED_COMBINED_LANE_MARKER_CONTRACT_STATUS",
    "GO_NO_GO_REQUIRED_PROMOTION_EVIDENCE_REASON_MARKERS_CSV",
    "GO_NO_GO_REQUIRED_PROMOTION_DECISION_REASON_MARKERS_CSV",
    "CI_TOOLS_FAST_MODE_REQUIRED_ENTRY",
    "CI_TOOLS_FAST_MODE_FORBIDDEN_ENTRY",
    "CI_FAST_GATE_WORKFLOW_FORBIDDEN_ENTRY",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--lifecycle-artifact-bundle-file", type=Path, required=True)
    parser.add_argument("--go-no-go-gate-report-file", type=Path, required=True)
    parser.add_argument("--threshold-file", type=Path, default=DEFAULT_THRESHOLD_FILE)
    parser.add_argument("--strategy-doc", type=Path, default=DEFAULT_STRATEGY_DOC)
    parser.add_argument("--ops-doc", type=Path, default=DEFAULT_OPS_DOC)
    parser.add_argument("--workflow-file", type=Path, default=DEFAULT_WORKFLOW_FILE)
    parser.add_argument("--ci-tools-file", type=Path, default=DEFAULT_CI_TOOLS_FILE)
    parser.add_argument("--output-json", type=Path)
    return parser.parse_args()


def parse_env(path: Path) -> dict[str, str]:
    values: dict[str, str] = {}
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            raise ValueError("invalid-env-line")
        key, value = line.split("=", 1)
        values[key.strip()] = value.strip()
    return values


def parse_csv(value: str) -> list[str]:
    entries = [entry.strip() for entry in value.split(",") if entry.strip()]
    if not entries:
        raise ValueError("expected-non-empty-csv")
    return entries


def parse_positive_int(value: str) -> int:
    parsed = int(value)
    if parsed <= 0:
        raise ValueError("expected-positive-int")
    return parsed


def parse_bool(value: str) -> bool:
    normalized = value.strip().lower()
    if normalized in {"1", "true", "yes"}:
        return True
    if normalized in {"0", "false", "no"}:
        return False
    raise ValueError("expected-bool")


def load_json(path: Path) -> dict:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("json-must-be-object")
    return payload


def extract_fast_mode_block(text: str) -> str:
    match = re.search(
        r'if \[ "\$\{KAMN_CI_TOOLS_FAST_MODE:-false\}" = "true" \]; then(?P<body>.*?)\n\s*echo "Fast-mode CI tool regression tests passed\."\n\s*exit 0\nfi',
        text,
        flags=re.DOTALL,
    )
    return "" if not match else match.group("body")


def lifecycle_bundle_ok(bundle_payload: dict, thresholds: dict[str, str]) -> bool:
    if bundle_payload.get("schema_version") != thresholds["LIFECYCLE_ARTIFACT_EXPECTED_SCHEMA_VERSION"]:
        return False
    if (
        bundle_payload.get("artifact_schema_version")
        != thresholds["LIFECYCLE_ARTIFACT_EXPECTED_ARTIFACT_SCHEMA_VERSION"]
    ):
        return False
    if (
        bundle_payload.get("reason_taxonomy_version")
        != thresholds["LIFECYCLE_ARTIFACT_EXPECTED_REASON_TAXONOMY_VERSION"]
    ):
        return False
    if bundle_payload.get("reason_codes_csv") != thresholds["LIFECYCLE_ARTIFACT_EXPECTED_REASON_CODES_CSV"]:
        return False
    if bundle_payload.get("final_decision") != thresholds["LIFECYCLE_ARTIFACT_EXPECTED_FINAL_DECISION"]:
        return False
    for hash_field in parse_csv(thresholds["LIFECYCLE_ARTIFACT_REQUIRED_HASH_FIELDS_CSV"]):
        value = bundle_payload.get(hash_field)
        if not isinstance(value, str) or not value.startswith("sha256:") or len(value) != 71:
            return False
    return True


def check_report_marker_csv(
    report_payload: dict,
    report_field: str,
    required_markers: list[str],
) -> bool:
    marker_csv = report_payload.get(report_field)
    return isinstance(marker_csv, str) and all(
        marker in marker_csv for marker in required_markers
    )


def go_no_go_report_ok(report_payload: dict, thresholds: dict[str, str]) -> tuple[bool, bool]:
    report_ok = True
    budget_ok = True

    if report_payload.get("schema_version") != thresholds["GO_NO_GO_EXPECTED_SCHEMA_VERSION"]:
        report_ok = False
    if (
        report_payload.get("reason_taxonomy_version")
        != thresholds["GO_NO_GO_EXPECTED_REASON_TAXONOMY_VERSION"]
    ):
        report_ok = False
    if report_payload.get("status") != thresholds["GO_NO_GO_EXPECTED_STATUS"]:
        report_ok = False
    if report_payload.get("final_decision") != thresholds["GO_NO_GO_EXPECTED_FINAL_DECISION"]:
        report_ok = False
    if report_payload.get("lane_mode") != thresholds["GO_NO_GO_EXPECTED_LANE_MODE"]:
        report_ok = False
    if report_payload.get("ci_fast_gate_eligible") is not parse_bool(
        thresholds["GO_NO_GO_EXPECTED_FAST_GATE_ELIGIBLE"]
    ):
        report_ok = False
    if report_payload.get("ci_fast_gate_scope") != thresholds["GO_NO_GO_EXPECTED_FAST_GATE_SCOPE"]:
        report_ok = False
    if (
        report_payload.get("fast_gate_exclusion_reason_code")
        != thresholds["GO_NO_GO_EXPECTED_FAST_GATE_EXCLUSION_REASON_CODE"]
    ):
        report_ok = False
    if (
        report_payload.get("run_mode_command_status")
        != thresholds["GO_NO_GO_EXPECTED_RUN_MODE_COMMAND_STATUS"]
    ):
        report_ok = False
    if (
        report_payload.get("combined_lane_marker_contract_status")
        != thresholds["GO_NO_GO_EXPECTED_COMBINED_LANE_MARKER_CONTRACT_STATUS"]
    ):
        report_ok = False

    if not check_report_marker_csv(
        report_payload,
        "promotion_evidence_reason_codes_csv",
        parse_csv(thresholds["GO_NO_GO_REQUIRED_PROMOTION_EVIDENCE_REASON_MARKERS_CSV"]),
    ):
        report_ok = False
    if not check_report_marker_csv(
        report_payload,
        "promotion_decision_reason_codes_csv",
        parse_csv(thresholds["GO_NO_GO_REQUIRED_PROMOTION_DECISION_REASON_MARKERS_CSV"]),
    ):
        report_ok = False

    max_seconds = parse_positive_int(thresholds["LIFECYCLE_CI_DRY_RUN_MAX_SECONDS"])
    elapsed = report_payload.get("elapsed_seconds")
    if not isinstance(elapsed, int) or isinstance(elapsed, bool) or elapsed < 0:
        report_ok = False
        budget_ok = False
    elif elapsed > max_seconds:
        budget_ok = False

    return report_ok, budget_ok


def doc_required_markers(thresholds: dict[str, str]) -> tuple[list[str], list[str], list[str]]:
    reason_codes_csv = thresholds["LIFECYCLE_CI_DRY_RUN_REASON_CODES_CSV"]
    checker_command = (
        "python3 scripts/ci/check_lifecycle_ci_dry_run_governance.py "
        "--lifecycle-artifact-bundle-file /tmp/lifecycle-artifact-integrity-baseline.json "
        "--go-no-go-gate-report-file /tmp/go-no-go-gate-report.json "
        "--threshold-file fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env "
        "--strategy-doc docs/ci/strategy.md "
        "--ops-doc docs/ops/configuration.md "
        "--workflow-file .github/workflows/ci-fast-gate.yml "
        "--ci-tools-file scripts/ci/test_ci_tools.sh "
        "--output-json /tmp/lifecycle-ci-dry-run-governance-report.json"
    )

    shared_markers = [
        f"lifecycle_ci_dry_run_reason_taxonomy_version={thresholds['LIFECYCLE_CI_DRY_RUN_REASON_TAXONOMY_VERSION']}",
        f"lifecycle_ci_dry_run_reason_codes_csv={reason_codes_csv}",
        "lifecycle_ci_dry_run_threshold_fixture_path=fixtures/ci/lifecycle_ci_dry_run_governance_thresholds.env",
        f"lifecycle_ci_dry_run_max_seconds={thresholds['LIFECYCLE_CI_DRY_RUN_MAX_SECONDS']}",
        "lifecycle_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture",
        "lifecycle_ci_dry_run_fast_mode_forbidden_entry=bash \"$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh\" --mode run",
        "lifecycle_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/run_go_no_go_gate_lane.sh --mode run",
        "lifecycle_ci_dry_run_remediation_map_version=v1",
    ]

    strategy_markers = [
        "### Lifecycle Artifact CI Dry-Run Governance Contract",
        checker_command,
        "cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture",
        "Regression: #4082",
        *shared_markers,
    ]
    ops_markers = [
        "## Lifecycle Artifact CI Dry-Run Governance Contract (Issue #4082)",
        checker_command,
        "cargo test -p kamn-core --test lifecycle_ci_dry_run_governance_contract -- --nocapture",
        "Regression: #4082",
        *shared_markers,
    ]
    return strategy_markers, ops_markers, parse_csv(reason_codes_csv)


def main() -> int:
    started = time.monotonic()
    args = parse_args()

    threshold_status = "verified"
    reports_status = "verified"
    selector_status = "verified"
    workflow_status = "verified"
    docs_status = "verified"
    docs_remediation_status = "verified"
    raw_reason_codes: list[str] = []

    thresholds: dict[str, str] = {}
    policy_schema_version = "unknown"
    reason_taxonomy_version = "unknown"
    reason_codes_csv = "unknown"
    max_seconds = "unknown"

    try:
        if not args.threshold_file.exists():
            raise FileNotFoundError("missing-threshold-file")
        thresholds = parse_env(args.threshold_file)
        missing = sorted(REQUIRED_THRESHOLD_KEYS - set(thresholds))
        if missing:
            raise KeyError(",".join(missing))

        policy_schema_version = thresholds["LIFECYCLE_CI_DRY_RUN_POLICY_SCHEMA_VERSION"]
        reason_taxonomy_version = thresholds["LIFECYCLE_CI_DRY_RUN_REASON_TAXONOMY_VERSION"]
        reason_codes_csv = thresholds["LIFECYCLE_CI_DRY_RUN_REASON_CODES_CSV"]
        max_seconds = thresholds["LIFECYCLE_CI_DRY_RUN_MAX_SECONDS"]

        _ = parse_positive_int(max_seconds)
        _ = parse_csv(reason_codes_csv)
        _ = parse_bool(thresholds["GO_NO_GO_EXPECTED_FAST_GATE_ELIGIBLE"])
    except Exception:
        threshold_status = "violation"
        raw_reason_codes.append("lifecycle_ci_dry_run_threshold_contract_violation")

    if thresholds:
        try:
            lifecycle_bundle_payload = load_json(args.lifecycle_artifact_bundle_file)
            go_no_go_payload = load_json(args.go_no_go_gate_report_file)

            lifecycle_ok = lifecycle_bundle_ok(lifecycle_bundle_payload, thresholds)
            go_no_go_ok, go_no_go_budget_ok = go_no_go_report_ok(go_no_go_payload, thresholds)

            if not lifecycle_ok:
                reports_status = "violation"
                raw_reason_codes.append("lifecycle_ci_dry_run_lifecycle_marker_parity_drift")
            if not go_no_go_ok:
                reports_status = "violation"
                raw_reason_codes.append("lifecycle_ci_dry_run_go_no_go_marker_parity_drift")
            if not (lifecycle_ok and go_no_go_ok):
                raw_reason_codes.append("lifecycle_ci_dry_run_report_contract_violation")
            if not go_no_go_budget_ok:
                raw_reason_codes.append("lifecycle_ci_dry_run_runtime_budget_exceeded")
        except Exception:
            reports_status = "violation"
            raw_reason_codes.append("lifecycle_ci_dry_run_report_contract_violation")

        try:
            ci_tools_text = args.ci_tools_file.read_text(encoding="utf-8")
            fast_mode_block = extract_fast_mode_block(ci_tools_text)
            required_entry = thresholds["CI_TOOLS_FAST_MODE_REQUIRED_ENTRY"]
            forbidden_entry = thresholds["CI_TOOLS_FAST_MODE_FORBIDDEN_ENTRY"]
            if not fast_mode_block or required_entry not in fast_mode_block:
                selector_status = "violation"
                raw_reason_codes.append("lifecycle_ci_dry_run_fast_mode_selector_drift")
            if forbidden_entry in fast_mode_block:
                selector_status = "violation"
                raw_reason_codes.append("lifecycle_ci_dry_run_fast_mode_selector_drift")
        except Exception:
            selector_status = "violation"
            raw_reason_codes.append("lifecycle_ci_dry_run_fast_mode_selector_drift")

        try:
            workflow_text = args.workflow_file.read_text(encoding="utf-8")
            if thresholds["CI_FAST_GATE_WORKFLOW_FORBIDDEN_ENTRY"] in workflow_text:
                workflow_status = "violation"
                raw_reason_codes.append("lifecycle_ci_dry_run_workflow_exclusion_drift")
        except Exception:
            workflow_status = "violation"
            raw_reason_codes.append("lifecycle_ci_dry_run_workflow_exclusion_drift")

        try:
            strategy_text = args.strategy_doc.read_text(encoding="utf-8")
            ops_text = args.ops_doc.read_text(encoding="utf-8")
            strategy_markers, ops_markers, reason_codes = doc_required_markers(thresholds)

            if any(marker not in strategy_text for marker in strategy_markers):
                docs_status = "violation"
                raw_reason_codes.append("lifecycle_ci_dry_run_docs_marker_parity_drift")
            if any(marker not in ops_text for marker in ops_markers):
                docs_status = "violation"
                raw_reason_codes.append("lifecycle_ci_dry_run_docs_marker_parity_drift")

            for reason_code in reason_codes:
                remediation_marker = f"lifecycle_ci_dry_run_remediation.{reason_code}="
                if remediation_marker not in strategy_text or remediation_marker not in ops_text:
                    docs_remediation_status = "violation"
                    raw_reason_codes.append(
                        "lifecycle_ci_dry_run_docs_remediation_marker_missing"
                    )
                    break
        except Exception:
            docs_status = "violation"
            docs_remediation_status = "violation"
            raw_reason_codes.append("lifecycle_ci_dry_run_docs_marker_parity_drift")

    elapsed_seconds = int(time.monotonic() - started)
    if max_seconds != "unknown" and elapsed_seconds > parse_positive_int(max_seconds):
        raw_reason_codes.append("lifecycle_ci_dry_run_runtime_budget_exceeded")

    ordered_reason_codes = (
        parse_csv(reason_codes_csv) if reason_codes_csv != "unknown" else FALLBACK_REASON_CODES
    )
    observed = set(raw_reason_codes)
    normalized_reason_codes = [code for code in ordered_reason_codes if code in observed]
    reason_codes_value = "none" if not normalized_reason_codes else ",".join(normalized_reason_codes)
    status = "pass" if not normalized_reason_codes else "fail"
    final_decision = "GO" if not normalized_reason_codes else "NO-GO"
    contract_status = "verified" if status == "pass" else "violation"

    payload = {
        "schema_version": policy_schema_version,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": reason_taxonomy_version,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes": normalized_reason_codes,
        "reason_codes_value": reason_codes_value,
        "lifecycle_ci_dry_run_contract_status": contract_status,
        "lifecycle_ci_dry_run_threshold_status": threshold_status,
        "lifecycle_ci_dry_run_reports_status": reports_status,
        "lifecycle_ci_dry_run_selector_status": selector_status,
        "lifecycle_ci_dry_run_workflow_status": workflow_status,
        "lifecycle_ci_dry_run_docs_status": docs_status,
        "lifecycle_ci_dry_run_docs_remediation_status": docs_remediation_status,
        "lifecycle_ci_dry_run_max_seconds": max_seconds,
        "lifecycle_ci_dry_run_elapsed_seconds": elapsed_seconds,
        "inputs": {
            "lifecycle_artifact_bundle_file": str(args.lifecycle_artifact_bundle_file),
            "go_no_go_gate_report_file": str(args.go_no_go_gate_report_file),
            "threshold_file": str(args.threshold_file),
            "strategy_doc": str(args.strategy_doc),
            "ops_doc": str(args.ops_doc),
            "workflow_file": str(args.workflow_file),
            "ci_tools_file": str(args.ci_tools_file),
        },
    }

    if args.output_json is not None:
        args.output_json.parent.mkdir(parents=True, exist_ok=True)
        args.output_json.write_text(
            json.dumps(payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={reason_taxonomy_version}")
    print(f"reason_codes_csv={reason_codes_csv}")
    print(f"reason_codes_value={reason_codes_value}")
    print(f"lifecycle_ci_dry_run_contract_status={contract_status}")
    print(f"lifecycle_ci_dry_run_threshold_status={threshold_status}")
    print(f"lifecycle_ci_dry_run_reports_status={reports_status}")
    print(f"lifecycle_ci_dry_run_selector_status={selector_status}")
    print(f"lifecycle_ci_dry_run_workflow_status={workflow_status}")
    print(f"lifecycle_ci_dry_run_docs_status={docs_status}")
    print(f"lifecycle_ci_dry_run_docs_remediation_status={docs_remediation_status}")
    print(f"lifecycle_ci_dry_run_max_seconds={max_seconds}")
    print(f"lifecycle_ci_dry_run_elapsed_seconds={elapsed_seconds}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
