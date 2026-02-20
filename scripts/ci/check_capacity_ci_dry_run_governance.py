#!/usr/bin/env python3
"""Fail-closed CI dry-run governance checker for capacity go/no-go parity."""

from __future__ import annotations

import argparse
import json
import re
import time
from pathlib import Path

DEFAULT_THRESHOLD_FILE = Path("fixtures/ci/capacity_ci_dry_run_governance_thresholds.env")
DEFAULT_STRATEGY_DOC = Path("docs/ci/strategy.md")
DEFAULT_WORKFLOW_FILE = Path(".github/workflows/ci-fast-gate.yml")
DEFAULT_CI_TOOLS_FILE = Path("scripts/ci/test_ci_tools.sh")

FALLBACK_REASON_CODES = [
    "capacity_ci_dry_run_argument_invalid",
    "capacity_ci_dry_run_threshold_contract_violation",
    "capacity_ci_dry_run_report_contract_violation",
    "capacity_ci_dry_run_go_no_go_marker_parity_drift",
    "capacity_ci_dry_run_performance_marker_parity_drift",
    "capacity_ci_dry_run_runtime_budget_exceeded",
    "capacity_ci_dry_run_fast_mode_selector_drift",
    "capacity_ci_dry_run_workflow_exclusion_drift",
    "capacity_ci_dry_run_docs_marker_parity_drift",
    "capacity_ci_dry_run_docs_remediation_marker_missing",
]

REQUIRED_THRESHOLD_KEYS = {
    "CAPACITY_CI_DRY_RUN_POLICY_SCHEMA_VERSION",
    "CAPACITY_CI_DRY_RUN_REASON_TAXONOMY_VERSION",
    "CAPACITY_CI_DRY_RUN_REASON_CODES_CSV",
    "CAPACITY_CI_DRY_RUN_MAX_SECONDS",
    "PERFORMANCE_SMOKE_EXPECTED_LANE",
    "PERFORMANCE_SMOKE_EXPECTED_WORKLOAD",
    "PERFORMANCE_SMOKE_REQUIRED_MARKERS_CSV",
    "GO_NO_GO_EXPECTED_SCHEMA_VERSION",
    "GO_NO_GO_EXPECTED_REASON_TAXONOMY_VERSION",
    "GO_NO_GO_EXPECTED_STATUS",
    "GO_NO_GO_EXPECTED_FINAL_DECISION",
    "GO_NO_GO_EXPECTED_LANE_MODE",
    "GO_NO_GO_EXPECTED_FAST_GATE_ELIGIBLE",
    "GO_NO_GO_EXPECTED_FAST_GATE_SCOPE",
    "GO_NO_GO_EXPECTED_FAST_GATE_EXCLUSION_REASON_CODE",
    "GO_NO_GO_EXPECTED_RUN_MODE_COMMAND_STATUS",
    "GO_NO_GO_REQUIRED_PROMOTION_DECISION_REASON_MARKERS_CSV",
    "GO_NO_GO_REQUIRED_PROMOTION_EVIDENCE_REASON_MARKERS_CSV",
    "CI_TOOLS_FAST_MODE_REQUIRED_ENTRY",
    "CI_TOOLS_FAST_MODE_FORBIDDEN_ENTRY",
    "CI_FAST_GATE_WORKFLOW_FORBIDDEN_ENTRY",
}


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--performance-report-file", type=Path, required=True)
    parser.add_argument("--go-no-go-gate-report-file", type=Path, required=True)
    parser.add_argument("--threshold-file", type=Path, default=DEFAULT_THRESHOLD_FILE)
    parser.add_argument("--strategy-doc", type=Path, default=DEFAULT_STRATEGY_DOC)
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
        raise ValueError("report-json-must-be-object")
    return payload


def extract_fast_mode_block(text: str) -> str:
    match = re.search(
        r'if \[ "\$\{KAMN_CI_TOOLS_FAST_MODE:-false\}" = "true" \]; then(?P<body>.*?)\n\s*echo "Fast-mode CI tool regression tests passed\."\n\s*exit 0\nfi',
        text,
        flags=re.DOTALL,
    )
    return "" if not match else match.group("body")


def check_report_marker_csv(
    report_payload: dict,
    report_field: str,
    required_markers: list[str],
) -> bool:
    marker_csv = report_payload.get(report_field)
    return isinstance(marker_csv, str) and all(
        marker in marker_csv for marker in required_markers
    )


def performance_report_ok(report_payload: dict, thresholds: dict[str, str]) -> bool:
    if report_payload.get("lane") != thresholds["PERFORMANCE_SMOKE_EXPECTED_LANE"]:
        return False
    if report_payload.get("workload") != thresholds["PERFORMANCE_SMOKE_EXPECTED_WORKLOAD"]:
        return False

    required_markers = parse_csv(thresholds["PERFORMANCE_SMOKE_REQUIRED_MARKERS_CSV"])
    if any(marker not in report_payload for marker in required_markers):
        return False

    for field in ("latency_p50_ms", "latency_p99_ms", "throughput_tps", "availability_pct"):
        value = report_payload.get(field)
        if not isinstance(value, (int, float)) or isinstance(value, bool) or value <= 0:
            return False
    return True


def go_no_go_report_ok(report_payload: dict, thresholds: dict[str, str]) -> bool:
    if report_payload.get("schema_version") != thresholds["GO_NO_GO_EXPECTED_SCHEMA_VERSION"]:
        return False
    if (
        report_payload.get("reason_taxonomy_version")
        != thresholds["GO_NO_GO_EXPECTED_REASON_TAXONOMY_VERSION"]
    ):
        return False
    if report_payload.get("status") != thresholds["GO_NO_GO_EXPECTED_STATUS"]:
        return False
    if report_payload.get("final_decision") != thresholds["GO_NO_GO_EXPECTED_FINAL_DECISION"]:
        return False
    if report_payload.get("lane_mode") != thresholds["GO_NO_GO_EXPECTED_LANE_MODE"]:
        return False
    if report_payload.get("ci_fast_gate_eligible") is not parse_bool(
        thresholds["GO_NO_GO_EXPECTED_FAST_GATE_ELIGIBLE"]
    ):
        return False
    if report_payload.get("ci_fast_gate_scope") != thresholds["GO_NO_GO_EXPECTED_FAST_GATE_SCOPE"]:
        return False
    if (
        report_payload.get("fast_gate_exclusion_reason_code")
        != thresholds["GO_NO_GO_EXPECTED_FAST_GATE_EXCLUSION_REASON_CODE"]
    ):
        return False
    if (
        report_payload.get("run_mode_command_status")
        != thresholds["GO_NO_GO_EXPECTED_RUN_MODE_COMMAND_STATUS"]
    ):
        return False

    if not check_report_marker_csv(
        report_payload,
        "promotion_decision_reason_codes_csv",
        parse_csv(thresholds["GO_NO_GO_REQUIRED_PROMOTION_DECISION_REASON_MARKERS_CSV"]),
    ):
        return False
    if not check_report_marker_csv(
        report_payload,
        "promotion_evidence_reason_codes_csv",
        parse_csv(thresholds["GO_NO_GO_REQUIRED_PROMOTION_EVIDENCE_REASON_MARKERS_CSV"]),
    ):
        return False
    return True


def doc_required_markers(thresholds: dict[str, str]) -> tuple[list[str], list[str]]:
    reason_codes_csv = thresholds["CAPACITY_CI_DRY_RUN_REASON_CODES_CSV"]
    markers = [
        "## Capacity CI Dry-Run Governance Contract",
        "python3 scripts/ci/check_capacity_ci_dry_run_governance.py --performance-report-file /tmp/performance-smoke-runtime-summary.json --go-no-go-gate-report-file /tmp/go-no-go-gate-report.json --threshold-file fixtures/ci/capacity_ci_dry_run_governance_thresholds.env --strategy-doc docs/ci/strategy.md --workflow-file .github/workflows/ci-fast-gate.yml --ci-tools-file scripts/ci/test_ci_tools.sh --output-json /tmp/capacity-ci-dry-run-governance-report.json",
        "cargo test -p kamn-core --test capacity_ci_dry_run_governance_contract -- --nocapture",
        "Regression: #4006",
        f"capacity_ci_dry_run_reason_taxonomy_version={thresholds['CAPACITY_CI_DRY_RUN_REASON_TAXONOMY_VERSION']}",
        f"capacity_ci_dry_run_reason_codes_csv={reason_codes_csv}",
        "capacity_ci_dry_run_threshold_fixture_path=fixtures/ci/capacity_ci_dry_run_governance_thresholds.env",
        f"capacity_ci_dry_run_max_seconds={thresholds['CAPACITY_CI_DRY_RUN_MAX_SECONDS']}",
        "capacity_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test capacity_ci_dry_run_governance_contract -- --nocapture",
        'capacity_ci_dry_run_fast_mode_forbidden_entry=bash "$ROOT_DIR/scripts/runtime/run_go_no_go_gate_lane.sh" --mode run',
        "capacity_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/run_go_no_go_gate_lane.sh --mode run",
        "capacity_ci_dry_run_remediation_map_version=v1",
    ]
    return markers, parse_csv(reason_codes_csv)


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

        policy_schema_version = thresholds["CAPACITY_CI_DRY_RUN_POLICY_SCHEMA_VERSION"]
        reason_taxonomy_version = thresholds["CAPACITY_CI_DRY_RUN_REASON_TAXONOMY_VERSION"]
        reason_codes_csv = thresholds["CAPACITY_CI_DRY_RUN_REASON_CODES_CSV"]
        max_seconds = thresholds["CAPACITY_CI_DRY_RUN_MAX_SECONDS"]
        _ = parse_positive_int(max_seconds)
        _ = parse_csv(reason_codes_csv)
        _ = parse_bool(thresholds["GO_NO_GO_EXPECTED_FAST_GATE_ELIGIBLE"])
    except Exception:
        threshold_status = "violation"
        raw_reason_codes.append("capacity_ci_dry_run_threshold_contract_violation")

    if thresholds:
        try:
            performance_payload = load_json(args.performance_report_file)
            go_no_go_payload = load_json(args.go_no_go_gate_report_file)
            perf_ok = performance_report_ok(performance_payload, thresholds)
            gonogo_ok = go_no_go_report_ok(go_no_go_payload, thresholds)

            if not perf_ok:
                reports_status = "violation"
                raw_reason_codes.append("capacity_ci_dry_run_performance_marker_parity_drift")
            if not gonogo_ok:
                reports_status = "violation"
                raw_reason_codes.append("capacity_ci_dry_run_go_no_go_marker_parity_drift")
            if not (perf_ok and gonogo_ok):
                raw_reason_codes.append("capacity_ci_dry_run_report_contract_violation")
        except Exception:
            reports_status = "violation"
            raw_reason_codes.append("capacity_ci_dry_run_report_contract_violation")

        try:
            ci_tools_text = args.ci_tools_file.read_text(encoding="utf-8")
            fast_mode = extract_fast_mode_block(ci_tools_text)
            required_entry = thresholds["CI_TOOLS_FAST_MODE_REQUIRED_ENTRY"]
            forbidden_entry = thresholds["CI_TOOLS_FAST_MODE_FORBIDDEN_ENTRY"]
            if not fast_mode or required_entry not in fast_mode or forbidden_entry in fast_mode:
                selector_status = "violation"
                raw_reason_codes.append("capacity_ci_dry_run_fast_mode_selector_drift")
        except Exception:
            selector_status = "violation"
            raw_reason_codes.append("capacity_ci_dry_run_fast_mode_selector_drift")

        try:
            workflow_text = args.workflow_file.read_text(encoding="utf-8")
            if thresholds["CI_FAST_GATE_WORKFLOW_FORBIDDEN_ENTRY"] in workflow_text:
                workflow_status = "violation"
                raw_reason_codes.append("capacity_ci_dry_run_workflow_exclusion_drift")
        except Exception:
            workflow_status = "violation"
            raw_reason_codes.append("capacity_ci_dry_run_workflow_exclusion_drift")

        try:
            strategy_text = args.strategy_doc.read_text(encoding="utf-8")
            markers, reason_codes = doc_required_markers(thresholds)
            if any(marker not in strategy_text for marker in markers):
                docs_status = "violation"
                raw_reason_codes.append("capacity_ci_dry_run_docs_marker_parity_drift")
            for reason_code in reason_codes:
                if f"capacity_ci_dry_run_remediation.{reason_code}=" not in strategy_text:
                    docs_remediation_status = "violation"
                    raw_reason_codes.append(
                        "capacity_ci_dry_run_docs_remediation_marker_missing"
                    )
                    break
        except Exception:
            docs_status = "violation"
            docs_remediation_status = "violation"
            raw_reason_codes.append("capacity_ci_dry_run_docs_marker_parity_drift")

    elapsed_seconds = int(time.monotonic() - started)
    if max_seconds != "unknown" and elapsed_seconds > parse_positive_int(max_seconds):
        raw_reason_codes.append("capacity_ci_dry_run_runtime_budget_exceeded")

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
        "capacity_ci_dry_run_contract_status": contract_status,
        "capacity_ci_dry_run_threshold_status": threshold_status,
        "capacity_ci_dry_run_reports_status": reports_status,
        "capacity_ci_dry_run_selector_status": selector_status,
        "capacity_ci_dry_run_workflow_status": workflow_status,
        "capacity_ci_dry_run_docs_status": docs_status,
        "capacity_ci_dry_run_docs_remediation_status": docs_remediation_status,
        "capacity_ci_dry_run_max_seconds": max_seconds,
        "capacity_ci_dry_run_elapsed_seconds": elapsed_seconds,
        "inputs": {
            "performance_report_file": str(args.performance_report_file),
            "go_no_go_gate_report_file": str(args.go_no_go_gate_report_file),
            "threshold_file": str(args.threshold_file),
            "strategy_doc": str(args.strategy_doc),
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
    print(f"capacity_ci_dry_run_contract_status={contract_status}")
    print(f"capacity_ci_dry_run_threshold_status={threshold_status}")
    print(f"capacity_ci_dry_run_reports_status={reports_status}")
    print(f"capacity_ci_dry_run_selector_status={selector_status}")
    print(f"capacity_ci_dry_run_workflow_status={workflow_status}")
    print(f"capacity_ci_dry_run_docs_status={docs_status}")
    print(f"capacity_ci_dry_run_docs_remediation_status={docs_remediation_status}")
    print(f"capacity_ci_dry_run_max_seconds={max_seconds}")
    print(f"capacity_ci_dry_run_elapsed_seconds={elapsed_seconds}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
