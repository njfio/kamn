#!/usr/bin/env python3
"""Policy checker for dependency local-heavy deep scan lane artifacts."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

RUNNER_REPORT_SCHEMA = "kamn.runtime.dependency-local-heavy-deep-scan-lane-report.v1"
RUNNER_ARTIFACT_SCHEMA = "kamn.runtime.dependency-local-heavy-deep-scan-artifact-schema.v1"
RUNNER_FIXTURE_SCHEMA = "kamn.ci.dependency-local-heavy-deep-scan-fixture-matrix.v1"
RUNNER_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.dependency-local-heavy-deep-scan-reason-taxonomy.v1"
)
RUNNER_REASON_CODES_CSV = (
    "dependency_local_heavy_deep_scan_profile_threshold_exceeded,"
    "dependency_local_heavy_deep_scan_runtime_budget_exceeded"
)

POLICY_REPORT_SCHEMA = "kamn.runtime.dependency-local-heavy-deep-scan-policy-report.v1"
POLICY_REASON_TAXONOMY_VERSION = (
    "kamn.runtime.dependency-local-heavy-deep-scan-policy-reason-taxonomy.v1"
)
POLICY_REASON_CODES_CSV = (
    "dependency_local_heavy_deep_scan_policy_required_field_missing,"
    "dependency_local_heavy_deep_scan_policy_marker_mismatch,"
    "dependency_local_heavy_deep_scan_policy_reason_taxonomy_mismatch,"
    "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch,"
    "dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch,"
    "dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_drift,"
    "dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_drift,"
    "ci_fast_gate_failed,"
    "dependency_local_heavy_deep_scan_policy_expected_decision_mismatch,"
    "dependency_local_heavy_deep_scan_policy_violation"
)
POLICY_REASON_CODES_ORDER = POLICY_REASON_CODES_CSV.split(",")

DEFAULT_STRATEGY_DOC = Path("docs/ci/strategy.md")
DEFAULT_OPS_DOC = Path("docs/ops/configuration.md")
DEFAULT_CI_TOOLS_FILE = Path("scripts/ci/test_ci_tools.sh")
DEFAULT_WORKFLOW_FILE = Path(".github/workflows/ci-fast-gate.yml")

REQUIRED_REPORT_FIELDS = (
    "schema_version",
    "artifact_schema_version",
    "fixture_schema_version",
    "reason_taxonomy_version",
    "reason_codes_csv",
    "status",
    "final_decision",
    "lane_mode",
    "profile",
    "profile_status",
    "reason_code",
    "reason_codes_value",
    "advisory_total",
    "critical_count",
    "high_count",
    "moderate_count",
    "low_count",
    "unknown_count",
    "threshold_max_critical",
    "threshold_max_high",
    "required_profiles_csv",
    "ci_fast_gate",
    "run_mode_command_status",
    "command_count",
    "performance_budget_status",
    "elapsed_seconds",
    "max_seconds",
)

STRATEGY_REQUIRED_MARKERS = (
    "## Dependency Local-Heavy Deep Scan Policy Checker Contract",
    "bash scripts/runtime/check_dependency_local_heavy_deep_scan_policy.sh --report-file "
    "/tmp/dependency-local-heavy-deep-scan-baseline.json --expected-final-decision GO "
    "--ci-fast-gate PASS --strategy-doc docs/ci/strategy.md --ops-doc "
    "docs/ops/configuration.md --ci-tools-file scripts/ci/test_ci_tools.sh --workflow-file "
    ".github/workflows/ci-fast-gate.yml --output-json "
    "/tmp/dependency-local-heavy-deep-scan-policy-report.json",
    "bash scripts/runtime/test_check_dependency_local_heavy_deep_scan_policy.sh",
    "dependency_local_heavy_deep_scan_policy_reason_taxonomy_version="
    "kamn.runtime.dependency-local-heavy-deep-scan-policy-reason-taxonomy.v1",
    "dependency_local_heavy_deep_scan_policy_reason_codes_csv="
    "dependency_local_heavy_deep_scan_policy_required_field_missing,"
    "dependency_local_heavy_deep_scan_policy_marker_mismatch,"
    "dependency_local_heavy_deep_scan_policy_reason_taxonomy_mismatch,"
    "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch,"
    "dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch,"
    "dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_drift,"
    "dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_drift,"
    "ci_fast_gate_failed,"
    "dependency_local_heavy_deep_scan_policy_expected_decision_mismatch,"
    "dependency_local_heavy_deep_scan_policy_violation",
    "dependency_local_heavy_deep_scan_policy_strategy_doc_path=docs/ci/strategy.md",
    "dependency_local_heavy_deep_scan_policy_ops_doc_path=docs/ops/configuration.md",
    "dependency_local_heavy_deep_scan_policy_runner_report_schema_version="
    "kamn.runtime.dependency-local-heavy-deep-scan-lane-report.v1",
    "dependency_local_heavy_deep_scan_policy_runner_reason_taxonomy_version="
    "kamn.runtime.dependency-local-heavy-deep-scan-reason-taxonomy.v1",
    "dependency_local_heavy_deep_scan_policy_runner_reason_codes_csv="
    "dependency_local_heavy_deep_scan_profile_threshold_exceeded,"
    "dependency_local_heavy_deep_scan_runtime_budget_exceeded",
    "dependency_local_heavy_deep_scan_policy_ci_dry_run_required_entry="
    "bash \"$ROOT_DIR/scripts/runtime/test_check_dependency_local_heavy_deep_scan_policy.sh\"",
    "dependency_local_heavy_deep_scan_policy_ci_dry_run_forbidden_entry="
    "bash \"$ROOT_DIR/scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh\" --profile baseline --mode run",
    "dependency_local_heavy_deep_scan_policy_workflow_forbidden_entry="
    "bash scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh --profile baseline --mode run",
)

OPS_REQUIRED_MARKERS = (
    "## Dependency Local-Heavy Deep Scan Lane Artifact Contract (Issue #4032)",
    "dependency_local_heavy_deep_scan_lane_schema_version="
    "kamn.runtime.dependency-local-heavy-deep-scan-lane-report.v1",
    "dependency_local_heavy_deep_scan_artifact_schema_version="
    "kamn.runtime.dependency-local-heavy-deep-scan-artifact-schema.v1",
    "dependency_local_heavy_deep_scan_fixture_schema_version="
    "kamn.ci.dependency-local-heavy-deep-scan-fixture-matrix.v1",
    "dependency_local_heavy_deep_scan_reason_taxonomy_version="
    "kamn.runtime.dependency-local-heavy-deep-scan-reason-taxonomy.v1",
    "dependency_local_heavy_deep_scan_reason_codes_csv="
    "dependency_local_heavy_deep_scan_profile_threshold_exceeded,"
    "dependency_local_heavy_deep_scan_runtime_budget_exceeded",
    "dependency_local_heavy_deep_scan_required_profiles_csv=baseline,injected-risk",
)

CI_DRY_RUN_REQUIRED_ENTRY = (
    'bash "$ROOT_DIR/scripts/runtime/test_check_dependency_local_heavy_deep_scan_policy.sh"'
)
CI_DRY_RUN_FORBIDDEN_ENTRY = (
    'bash "$ROOT_DIR/scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh" '
    "--profile baseline --mode run"
)
WORKFLOW_FORBIDDEN_ENTRY = (
    "bash scripts/runtime/run_dependency_local_heavy_deep_scan_lane.sh --profile baseline "
    "--mode run"
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
    parser.add_argument("--ci-tools-file", type=Path, default=DEFAULT_CI_TOOLS_FILE)
    parser.add_argument("--workflow-file", type=Path, default=DEFAULT_WORKFLOW_FILE)
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


def evaluate_marker_parity(doc_path: Path, required_markers: tuple[str, ...]) -> str:
    try:
        text = doc_path.read_text(encoding="utf-8")
    except OSError:
        return "violation"

    missing_markers = [marker for marker in required_markers if marker not in text]
    return "violation" if missing_markers else "verified"


def extract_fast_mode_block(text: str) -> str:
    match = re.search(
        r'if \[ "\$\{KAMN_CI_TOOLS_FAST_MODE:-false\}" = "true" \]; then(?P<body>.*?)\n\s*echo "Fast-mode CI tool regression tests passed\."\n\s*exit 0\nfi',
        text,
        flags=re.DOTALL,
    )
    return "" if not match else match.group("body")


def expected_profile_markers(
    profile: str,
) -> tuple[str, str, str, int, int, str]:
    if profile == "baseline":
        return ("pass", "GO", "verified", 0, 0, "none")
    if profile == "injected-risk":
        return (
            "fail",
            "NO-GO",
            "failed",
            1,
            2,
            "dependency_local_heavy_deep_scan_profile_threshold_exceeded",
        )
    return ("invalid", "INVALID", "invalid", -1, -1, "invalid")


def main() -> int:
    args = parse_args()

    raw_reason_codes: list[str] = []
    marker_status = "verified"
    reason_taxonomy_status = "verified"
    profile_contract_status = "verified"

    if not args.report_file.exists():
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_required_field_missing"
        )
        payload: dict[str, object] = {}
    else:
        try:
            payload = load_json_dict(args.report_file)
        except Exception:
            payload = {}
            raw_reason_codes.append(
                "dependency_local_heavy_deep_scan_policy_marker_mismatch"
            )

    missing_fields = [
        field_name for field_name in REQUIRED_REPORT_FIELDS if field_name not in payload
    ]
    if missing_fields:
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_required_field_missing"
        )

    if payload.get("schema_version") != RUNNER_REPORT_SCHEMA:
        marker_status = "violation"
        raw_reason_codes.append("dependency_local_heavy_deep_scan_policy_marker_mismatch")
    if payload.get("artifact_schema_version") != RUNNER_ARTIFACT_SCHEMA:
        marker_status = "violation"
        raw_reason_codes.append("dependency_local_heavy_deep_scan_policy_marker_mismatch")
    if payload.get("fixture_schema_version") != RUNNER_FIXTURE_SCHEMA:
        marker_status = "violation"
        raw_reason_codes.append("dependency_local_heavy_deep_scan_policy_marker_mismatch")

    if payload.get("reason_taxonomy_version") != RUNNER_REASON_TAXONOMY_VERSION:
        reason_taxonomy_status = "violation"
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_reason_taxonomy_mismatch"
        )
    if payload.get("reason_codes_csv") != RUNNER_REASON_CODES_CSV:
        reason_taxonomy_status = "violation"
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_reason_taxonomy_mismatch"
        )

    if payload.get("ci_fast_gate") != args.ci_fast_gate:
        raw_reason_codes.append("ci_fast_gate_failed")
    if args.ci_fast_gate != "PASS":
        raw_reason_codes.append("ci_fast_gate_failed")

    lane_mode = payload.get("lane_mode")
    if lane_mode not in ("dry-run", "run"):
        profile_contract_status = "violation"
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"
        )

    profile = payload.get("profile")
    if profile not in ("baseline", "injected-risk"):
        profile_contract_status = "violation"
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"
        )
    else:
        (
            expected_status,
            expected_final_decision,
            expected_profile_status,
            expected_critical,
            expected_high,
            expected_reason_code,
        ) = expected_profile_markers(profile)
        if payload.get("status") != expected_status:
            profile_contract_status = "violation"
            raw_reason_codes.append(
                "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"
            )
        if payload.get("final_decision") != expected_final_decision:
            profile_contract_status = "violation"
            raw_reason_codes.append(
                "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"
            )
        if payload.get("profile_status") != expected_profile_status:
            profile_contract_status = "violation"
            raw_reason_codes.append(
                "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"
            )
        if payload.get("critical_count") != expected_critical:
            profile_contract_status = "violation"
            raw_reason_codes.append(
                "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"
            )
        if payload.get("high_count") != expected_high:
            profile_contract_status = "violation"
            raw_reason_codes.append(
                "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"
            )
        if payload.get("reason_code") != expected_reason_code:
            profile_contract_status = "violation"
            raw_reason_codes.append(
                "dependency_local_heavy_deep_scan_policy_profile_contract_mismatch"
            )

    strategy_parity_status = evaluate_marker_parity(args.strategy_doc, STRATEGY_REQUIRED_MARKERS)
    ops_parity_status = evaluate_marker_parity(args.ops_doc, OPS_REQUIRED_MARKERS)
    docs_marker_parity_status = (
        "verified"
        if strategy_parity_status == "verified" and ops_parity_status == "verified"
        else "violation"
    )
    if docs_marker_parity_status != "verified":
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_docs_marker_parity_mismatch"
        )

    selector_status = "verified"
    try:
        ci_tools_text = args.ci_tools_file.read_text(encoding="utf-8")
        fast_mode_block = extract_fast_mode_block(ci_tools_text)
        if CI_DRY_RUN_REQUIRED_ENTRY not in fast_mode_block:
            selector_status = "violation"
        if CI_DRY_RUN_FORBIDDEN_ENTRY in fast_mode_block:
            selector_status = "violation"
    except OSError:
        selector_status = "violation"

    if selector_status != "verified":
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_drift"
        )

    workflow_status = "verified"
    try:
        workflow_text = args.workflow_file.read_text(encoding="utf-8")
        if WORKFLOW_FORBIDDEN_ENTRY in workflow_text:
            workflow_status = "violation"
    except OSError:
        workflow_status = "violation"

    if workflow_status != "verified":
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_drift"
        )

    if payload.get("final_decision") != args.expected_final_decision:
        raw_reason_codes.append(
            "dependency_local_heavy_deep_scan_policy_expected_decision_mismatch"
        )

    normalized_reason_codes = normalize_reason_codes(raw_reason_codes)
    if raw_reason_codes and not normalized_reason_codes:
        normalized_reason_codes = ["dependency_local_heavy_deep_scan_policy_violation"]

    status = "pass" if not normalized_reason_codes else "fail"
    final_decision = "GO" if status == "pass" else "NO-GO"

    if final_decision != args.expected_final_decision:
        if (
            "dependency_local_heavy_deep_scan_policy_expected_decision_mismatch"
            not in normalized_reason_codes
        ):
            normalized_reason_codes.append(
                "dependency_local_heavy_deep_scan_policy_expected_decision_mismatch"
            )
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
        "dependency_local_heavy_deep_scan_policy_status": policy_status,
        "dependency_local_heavy_deep_scan_policy_marker_status": marker_status,
        "dependency_local_heavy_deep_scan_policy_reason_taxonomy_status": reason_taxonomy_status,
        "dependency_local_heavy_deep_scan_policy_profile_contract_status": profile_contract_status,
        "dependency_local_heavy_deep_scan_policy_docs_marker_parity_status": docs_marker_parity_status,
        "dependency_local_heavy_deep_scan_policy_strategy_marker_parity_status": strategy_parity_status,
        "dependency_local_heavy_deep_scan_policy_ops_marker_parity_status": ops_parity_status,
        "dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_status": selector_status,
        "dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_status": workflow_status,
        "promotion_decision_reason_mapping_status": promotion_reason_mapping_status,
        "promotion_decision_reason_code": promotion_reason_code,
        "inputs": {
            "report_file": str(args.report_file),
            "expected_final_decision": args.expected_final_decision,
            "ci_fast_gate": args.ci_fast_gate,
            "strategy_doc": str(args.strategy_doc),
            "ops_doc": str(args.ops_doc),
            "ci_tools_file": str(args.ci_tools_file),
            "workflow_file": str(args.workflow_file),
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
    print(f"dependency_local_heavy_deep_scan_policy_status={policy_status}")
    print(f"dependency_local_heavy_deep_scan_policy_marker_status={marker_status}")
    print(
        "dependency_local_heavy_deep_scan_policy_reason_taxonomy_status="
        f"{reason_taxonomy_status}"
    )
    print(
        "dependency_local_heavy_deep_scan_policy_profile_contract_status="
        f"{profile_contract_status}"
    )
    print(
        "dependency_local_heavy_deep_scan_policy_docs_marker_parity_status="
        f"{docs_marker_parity_status}"
    )
    print(
        "dependency_local_heavy_deep_scan_policy_strategy_marker_parity_status="
        f"{strategy_parity_status}"
    )
    print(
        "dependency_local_heavy_deep_scan_policy_ops_marker_parity_status="
        f"{ops_parity_status}"
    )
    print(
        "dependency_local_heavy_deep_scan_policy_ci_dry_run_selector_status="
        f"{selector_status}"
    )
    print(
        "dependency_local_heavy_deep_scan_policy_ci_dry_run_workflow_status="
        f"{workflow_status}"
    )
    print(f"promotion_decision_reason_mapping_status={promotion_reason_mapping_status}")
    print(f"promotion_decision_reason_code={promotion_reason_code}")

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
