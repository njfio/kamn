#!/usr/bin/env python3
"""Fail-closed CI dry-run durability governance checker for sqlite crash-recovery."""

from __future__ import annotations

import argparse
import json
import re
from dataclasses import dataclass
from pathlib import Path


@dataclass
class CheckInputs:
    sqlite_crash_recovery_summary_report_file: Path
    sqlite_crash_recovery_policy_report_file: Path
    sqlite_crash_recovery_contract_lane_report_file: Path
    threshold_file: Path
    strategy_doc: Path
    ops_doc: Path
    workflow_file: Path
    ci_tools_file: Path
    output_json: Path | None


DEFAULT_THRESHOLD_FILE = Path(
    "fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env"
)
DEFAULT_STRATEGY_DOC = Path("docs/ci/strategy.md")
DEFAULT_OPS_DOC = Path("docs/ops/configuration.md")
DEFAULT_WORKFLOW_FILE = Path(".github/workflows/ci-fast-gate.yml")
DEFAULT_CI_TOOLS_FILE = Path("scripts/ci/test_ci_tools.sh")


def parse_args() -> CheckInputs:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--sqlite-crash-recovery-summary-report-file",
        type=Path,
        required=True,
    )
    parser.add_argument(
        "--sqlite-crash-recovery-policy-report-file",
        type=Path,
        required=True,
    )
    parser.add_argument(
        "--sqlite-crash-recovery-contract-lane-report-file",
        type=Path,
        required=True,
    )
    parser.add_argument("--threshold-file", type=Path, default=DEFAULT_THRESHOLD_FILE)
    parser.add_argument("--strategy-doc", type=Path, default=DEFAULT_STRATEGY_DOC)
    parser.add_argument("--ops-doc", type=Path, default=DEFAULT_OPS_DOC)
    parser.add_argument("--workflow-file", type=Path, default=DEFAULT_WORKFLOW_FILE)
    parser.add_argument("--ci-tools-file", type=Path, default=DEFAULT_CI_TOOLS_FILE)
    parser.add_argument("--output-json", type=Path)
    args = parser.parse_args()
    return CheckInputs(
        sqlite_crash_recovery_summary_report_file=args.sqlite_crash_recovery_summary_report_file,
        sqlite_crash_recovery_policy_report_file=args.sqlite_crash_recovery_policy_report_file,
        sqlite_crash_recovery_contract_lane_report_file=args.sqlite_crash_recovery_contract_lane_report_file,
        threshold_file=args.threshold_file,
        strategy_doc=args.strategy_doc,
        ops_doc=args.ops_doc,
        workflow_file=args.workflow_file,
        ci_tools_file=args.ci_tools_file,
        output_json=args.output_json,
    )


def parse_env_file(path: Path) -> dict[str, str]:
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


def parse_positive_int(value: str) -> int:
    parsed = int(value)
    if parsed <= 0:
        raise ValueError("expected-positive-int")
    return parsed


def parse_csv(value: str) -> list[str]:
    entries = [entry.strip() for entry in value.split(",") if entry.strip()]
    if not entries:
        raise ValueError("expected-non-empty-csv")
    return entries


def load_json(path: Path) -> dict:
    payload = json.loads(path.read_text(encoding="utf-8"))
    if not isinstance(payload, dict):
        raise ValueError("report-json-must-be-object")
    return payload


def extract_fast_mode_block(text: str) -> str:
    fast_mode_match = re.search(
        r'if \[ "\$\{KAMN_CI_TOOLS_FAST_MODE:-false\}" = "true" \]; then(?P<body>.*?)\n\s*echo "Fast-mode CI tool regression tests passed\."\n\s*exit 0\nfi',
        text,
        flags=re.DOTALL,
    )
    if not fast_mode_match:
        return ""
    return fast_mode_match.group("body")


def normalize_reason_codes(raw: list[str], ordered: list[str]) -> list[str]:
    observed = set(raw)
    return [code for code in ordered if code in observed]


def reason_codes_value(reason_codes: list[str]) -> str:
    return "none" if not reason_codes else ",".join(reason_codes)


def report_contains_required_markers(
    report_payload: dict,
    required_markers: list[str],
) -> bool:
    report_text = json.dumps(report_payload, sort_keys=True)
    for marker in required_markers:
        if "=" in marker:
            key, expected = marker.split("=", 1)
            observed = report_payload.get(key)
            if str(observed) != expected:
                return False
        elif marker not in report_text:
            return False
    return True


def check_report_contract(
    *,
    report_payload: dict,
    expected_schema: str,
    expected_status: str,
    required_markers: list[str],
    expected_exact_fields: dict[str, str],
    max_elapsed_seconds: int | None,
) -> tuple[bool, bool]:
    contract_ok = True
    budget_ok = True

    if report_payload.get("schema_version") != expected_schema:
        contract_ok = False
    if report_payload.get("status") != expected_status:
        contract_ok = False
    if report_payload.get("final_decision") != "GO":
        contract_ok = False
    if not report_contains_required_markers(report_payload, required_markers):
        contract_ok = False

    for key, expected in expected_exact_fields.items():
        if report_payload.get(key) != expected:
            contract_ok = False

    if max_elapsed_seconds is not None:
        elapsed_seconds = report_payload.get("elapsed_seconds")
        if (
            not isinstance(elapsed_seconds, int)
            or isinstance(elapsed_seconds, bool)
            or elapsed_seconds < 0
        ):
            contract_ok = False
            budget_ok = False
        elif elapsed_seconds > max_elapsed_seconds:
            budget_ok = False

    return contract_ok, budget_ok


def build_doc_required_markers(
    thresholds: dict[str, str],
    reason_codes_csv: str,
) -> tuple[list[str], list[str], list[str]]:
    checker_command = (
        "python3 scripts/ci/check_sqlite_crash_recovery_ci_dry_run_governance.py "
        "--sqlite-crash-recovery-summary-report-file /tmp/sqlite-crash-recovery-live-summary.json "
        "--sqlite-crash-recovery-policy-report-file /tmp/sqlite-crash-recovery-live-policy.json "
        "--sqlite-crash-recovery-contract-lane-report-file /tmp/sqlite-crash-recovery-live-contract-lane-report.json "
        "--threshold-file fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env "
        "--strategy-doc docs/ci/strategy.md "
        "--ops-doc docs/ops/configuration.md "
        "--workflow-file .github/workflows/ci-fast-gate.yml "
        "--ci-tools-file scripts/ci/test_ci_tools.sh "
        "--output-json /tmp/sqlite-crash-recovery-ci-dry-run-governance-report.json"
    )
    contract_test_command = (
        "cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract -- --nocapture"
    )

    shared_markers = [
        f"sqlite_crash_recovery_ci_dry_run_reason_taxonomy_version={thresholds['SQLITE_CRASH_RECOVERY_CI_DRY_RUN_REASON_TAXONOMY_VERSION']}",
        f"sqlite_crash_recovery_ci_dry_run_reason_codes_csv={reason_codes_csv}",
        "sqlite_crash_recovery_ci_dry_run_threshold_fixture_path=fixtures/ci/sqlite_crash_recovery_ci_dry_run_governance_thresholds.env",
        f"sqlite_crash_recovery_ci_dry_run_max_seconds={thresholds['SQLITE_CRASH_RECOVERY_CI_DRY_RUN_MAX_SECONDS']}",
        "sqlite_crash_recovery_ci_dry_run_fast_mode_required_entry=cargo test -p kamn-core --test sqlite_crash_recovery_ci_dry_run_governance_contract -- --nocapture",
        "sqlite_crash_recovery_ci_dry_run_fast_mode_forbidden_entry=bash \"$ROOT_DIR/scripts/runtime/validate_sqlite_crash_recovery_live.sh\" --mode run",
        "sqlite_crash_recovery_ci_dry_run_workflow_forbidden_entry=bash scripts/runtime/validate_sqlite_crash_recovery_live.sh --mode run",
        "sqlite_crash_recovery_ci_dry_run_remediation_map_version=v1",
    ]

    strategy_markers = [
        "## SQLite Crash-Recovery CI Dry-Run Durability Governance Contract",
        checker_command,
        contract_test_command,
        "Regression: #4014",
        *shared_markers,
    ]
    ops_markers = [
        "## SQLite Crash-Recovery CI Dry-Run Durability Governance Contract (Issue #4014)",
        checker_command,
        contract_test_command,
        "Regression: #4014",
        *shared_markers,
    ]
    return strategy_markers, ops_markers, parse_csv(reason_codes_csv)


def main() -> int:
    args = parse_args()

    raw_reason_codes: list[str] = []
    threshold_status = "verified"
    reports_status = "verified"
    selector_status = "verified"
    workflow_status = "verified"
    docs_status = "verified"
    docs_remediation_status = "verified"

    thresholds: dict[str, str] = {}
    policy_schema_version = "unknown"
    reason_taxonomy_version = "unknown"
    reason_codes_csv = "unknown"
    ordered_reason_codes: list[str] = []

    try:
        if not args.threshold_file.exists():
            raise FileNotFoundError("threshold file missing")
        thresholds = parse_env_file(args.threshold_file)
        required_threshold_keys = {
            "SQLITE_CRASH_RECOVERY_CI_DRY_RUN_POLICY_SCHEMA_VERSION",
            "SQLITE_CRASH_RECOVERY_CI_DRY_RUN_REASON_TAXONOMY_VERSION",
            "SQLITE_CRASH_RECOVERY_CI_DRY_RUN_REASON_CODES_CSV",
            "SQLITE_CRASH_RECOVERY_CI_DRY_RUN_MAX_SECONDS",
            "SQLITE_CRASH_RECOVERY_SUMMARY_EXPECTED_SCHEMA_VERSION",
            "SQLITE_CRASH_RECOVERY_SUMMARY_EXPECTED_STATUS",
            "SQLITE_CRASH_RECOVERY_SUMMARY_REQUIRED_MARKERS_CSV",
            "SQLITE_CRASH_RECOVERY_SUMMARY_EXPECTED_DURABILITY_REASON_CODES_CSV",
            "SQLITE_CRASH_RECOVERY_SUMMARY_MAX_ELAPSED_SECONDS",
            "SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_SCHEMA_VERSION",
            "SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_STATUS",
            "SQLITE_CRASH_RECOVERY_POLICY_REQUIRED_MARKERS_CSV",
            "SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_PROMOTION_REASON_CODES_CSV",
            "SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_DURABILITY_REASON_CODES_CSV",
            "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_SCHEMA_VERSION",
            "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_STATUS",
            "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_REQUIRED_MARKERS_CSV",
            "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_EVIDENCE_REASON_CODES_CSV",
            "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_PROMOTION_REASON_CODES_CSV",
            "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_MAX_ELAPSED_SECONDS",
            "CI_TOOLS_FAST_MODE_REQUIRED_ENTRY",
            "CI_TOOLS_FAST_MODE_FORBIDDEN_ENTRY",
            "CI_FAST_GATE_WORKFLOW_FORBIDDEN_ENTRY",
        }
        missing_threshold_keys = sorted(required_threshold_keys - set(thresholds))
        if missing_threshold_keys:
            raise KeyError(
                "missing threshold keys: " + ",".join(missing_threshold_keys)
            )

        policy_schema_version = thresholds[
            "SQLITE_CRASH_RECOVERY_CI_DRY_RUN_POLICY_SCHEMA_VERSION"
        ]
        reason_taxonomy_version = thresholds[
            "SQLITE_CRASH_RECOVERY_CI_DRY_RUN_REASON_TAXONOMY_VERSION"
        ]
        reason_codes_csv = thresholds["SQLITE_CRASH_RECOVERY_CI_DRY_RUN_REASON_CODES_CSV"]
        ordered_reason_codes = parse_csv(reason_codes_csv)
        _ = parse_positive_int(thresholds["SQLITE_CRASH_RECOVERY_CI_DRY_RUN_MAX_SECONDS"])
        _ = parse_positive_int(thresholds["SQLITE_CRASH_RECOVERY_SUMMARY_MAX_ELAPSED_SECONDS"])
        _ = parse_positive_int(
            thresholds["SQLITE_CRASH_RECOVERY_CONTRACT_LANE_MAX_ELAPSED_SECONDS"]
        )
        _ = parse_csv(thresholds["SQLITE_CRASH_RECOVERY_SUMMARY_REQUIRED_MARKERS_CSV"])
        _ = parse_csv(thresholds["SQLITE_CRASH_RECOVERY_POLICY_REQUIRED_MARKERS_CSV"])
        _ = parse_csv(thresholds["SQLITE_CRASH_RECOVERY_CONTRACT_LANE_REQUIRED_MARKERS_CSV"])
        _ = parse_csv(
            thresholds["SQLITE_CRASH_RECOVERY_SUMMARY_EXPECTED_DURABILITY_REASON_CODES_CSV"]
        )
        _ = parse_csv(
            thresholds["SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_PROMOTION_REASON_CODES_CSV"]
        )
        _ = parse_csv(
            thresholds["SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_DURABILITY_REASON_CODES_CSV"]
        )
        _ = parse_csv(
            thresholds[
                "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_EVIDENCE_REASON_CODES_CSV"
            ]
        )
        _ = parse_csv(
            thresholds[
                "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_PROMOTION_REASON_CODES_CSV"
            ]
        )
    except Exception:
        threshold_status = "violation"
        raw_reason_codes.append("sqlite_crash_recovery_ci_dry_run_threshold_contract_violation")

    if thresholds:
        try:
            summary_report = load_json(args.sqlite_crash_recovery_summary_report_file)
            policy_report = load_json(args.sqlite_crash_recovery_policy_report_file)
            contract_lane_report = load_json(
                args.sqlite_crash_recovery_contract_lane_report_file
            )

            summary_ok, summary_budget_ok = check_report_contract(
                report_payload=summary_report,
                expected_schema=thresholds[
                    "SQLITE_CRASH_RECOVERY_SUMMARY_EXPECTED_SCHEMA_VERSION"
                ],
                expected_status=thresholds["SQLITE_CRASH_RECOVERY_SUMMARY_EXPECTED_STATUS"],
                required_markers=parse_csv(
                    thresholds["SQLITE_CRASH_RECOVERY_SUMMARY_REQUIRED_MARKERS_CSV"]
                ),
                expected_exact_fields={
                    "durability_governance_reason_codes_csv": thresholds[
                        "SQLITE_CRASH_RECOVERY_SUMMARY_EXPECTED_DURABILITY_REASON_CODES_CSV"
                    ],
                },
                max_elapsed_seconds=parse_positive_int(
                    thresholds["SQLITE_CRASH_RECOVERY_SUMMARY_MAX_ELAPSED_SECONDS"]
                ),
            )

            policy_ok, policy_budget_ok = check_report_contract(
                report_payload=policy_report,
                expected_schema=thresholds[
                    "SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_SCHEMA_VERSION"
                ],
                expected_status=thresholds["SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_STATUS"],
                required_markers=parse_csv(
                    thresholds["SQLITE_CRASH_RECOVERY_POLICY_REQUIRED_MARKERS_CSV"]
                ),
                expected_exact_fields={
                    "promotion_decision_reason_codes_csv": thresholds[
                        "SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_PROMOTION_REASON_CODES_CSV"
                    ],
                    "durability_governance_reason_codes_csv": thresholds[
                        "SQLITE_CRASH_RECOVERY_POLICY_EXPECTED_DURABILITY_REASON_CODES_CSV"
                    ],
                },
                max_elapsed_seconds=None,
            )

            contract_lane_ok, contract_lane_budget_ok = check_report_contract(
                report_payload=contract_lane_report,
                expected_schema=thresholds[
                    "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_SCHEMA_VERSION"
                ],
                expected_status=thresholds[
                    "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_STATUS"
                ],
                required_markers=parse_csv(
                    thresholds["SQLITE_CRASH_RECOVERY_CONTRACT_LANE_REQUIRED_MARKERS_CSV"]
                ),
                expected_exact_fields={
                    "sqlite_crash_replay_evidence_reason_codes_csv": thresholds[
                        "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_EVIDENCE_REASON_CODES_CSV"
                    ],
                    "promotion_decision_reason_codes_csv": thresholds[
                        "SQLITE_CRASH_RECOVERY_CONTRACT_LANE_EXPECTED_PROMOTION_REASON_CODES_CSV"
                    ],
                },
                max_elapsed_seconds=parse_positive_int(
                    thresholds["SQLITE_CRASH_RECOVERY_CONTRACT_LANE_MAX_ELAPSED_SECONDS"]
                ),
            )

            if not (summary_ok and policy_ok and contract_lane_ok):
                reports_status = "violation"
                raw_reason_codes.append(
                    "sqlite_crash_recovery_ci_dry_run_report_contract_violation"
                )
            if not (summary_budget_ok and policy_budget_ok and contract_lane_budget_ok):
                raw_reason_codes.append(
                    "sqlite_crash_recovery_ci_dry_run_runtime_budget_exceeded"
                )
        except Exception:
            reports_status = "violation"
            raw_reason_codes.append(
                "sqlite_crash_recovery_ci_dry_run_report_contract_violation"
            )

        try:
            ci_tools_text = args.ci_tools_file.read_text(encoding="utf-8")
            fast_mode_block = extract_fast_mode_block(ci_tools_text)
            required_entry = thresholds["CI_TOOLS_FAST_MODE_REQUIRED_ENTRY"]
            forbidden_entry = thresholds["CI_TOOLS_FAST_MODE_FORBIDDEN_ENTRY"]
            if not fast_mode_block or required_entry not in fast_mode_block:
                selector_status = "violation"
                raw_reason_codes.append(
                    "sqlite_crash_recovery_ci_dry_run_fast_mode_selector_drift"
                )
            if forbidden_entry in fast_mode_block:
                selector_status = "violation"
                raw_reason_codes.append(
                    "sqlite_crash_recovery_ci_dry_run_fast_mode_selector_drift"
                )
        except Exception:
            selector_status = "violation"
            raw_reason_codes.append(
                "sqlite_crash_recovery_ci_dry_run_fast_mode_selector_drift"
            )

        try:
            workflow_text = args.workflow_file.read_text(encoding="utf-8")
            if thresholds["CI_FAST_GATE_WORKFLOW_FORBIDDEN_ENTRY"] in workflow_text:
                workflow_status = "violation"
                raw_reason_codes.append(
                    "sqlite_crash_recovery_ci_dry_run_workflow_exclusion_drift"
                )
        except Exception:
            workflow_status = "violation"
            raw_reason_codes.append(
                "sqlite_crash_recovery_ci_dry_run_workflow_exclusion_drift"
            )

        try:
            strategy_text = args.strategy_doc.read_text(encoding="utf-8")
            ops_text = args.ops_doc.read_text(encoding="utf-8")
            strategy_markers, ops_markers, checker_reason_codes = build_doc_required_markers(
                thresholds,
                thresholds["SQLITE_CRASH_RECOVERY_CI_DRY_RUN_REASON_CODES_CSV"],
            )
            if any(marker not in strategy_text for marker in strategy_markers):
                docs_status = "violation"
                raw_reason_codes.append(
                    "sqlite_crash_recovery_ci_dry_run_docs_marker_parity_drift"
                )
            if any(marker not in ops_text for marker in ops_markers):
                docs_status = "violation"
                raw_reason_codes.append(
                    "sqlite_crash_recovery_ci_dry_run_docs_marker_parity_drift"
                )

            for reason_code in checker_reason_codes:
                strategy_remediation_marker = (
                    f"sqlite_crash_recovery_ci_dry_run_remediation.{reason_code}="
                )
                ops_remediation_marker = (
                    f"sqlite_crash_recovery_ci_dry_run_remediation.{reason_code}="
                )
                if (
                    strategy_remediation_marker not in strategy_text
                    or ops_remediation_marker not in ops_text
                ):
                    docs_remediation_status = "violation"
                    raw_reason_codes.append(
                        "sqlite_crash_recovery_ci_dry_run_docs_remediation_marker_missing"
                    )
                    break
        except Exception:
            docs_status = "violation"
            docs_remediation_status = "violation"
            raw_reason_codes.append(
                "sqlite_crash_recovery_ci_dry_run_docs_marker_parity_drift"
            )

    ordered_reason_codes = (
        ordered_reason_codes
        if ordered_reason_codes
        else [
            "sqlite_crash_recovery_ci_dry_run_argument_invalid",
            "sqlite_crash_recovery_ci_dry_run_threshold_contract_violation",
            "sqlite_crash_recovery_ci_dry_run_report_contract_violation",
            "sqlite_crash_recovery_ci_dry_run_runtime_budget_exceeded",
            "sqlite_crash_recovery_ci_dry_run_fast_mode_selector_drift",
            "sqlite_crash_recovery_ci_dry_run_workflow_exclusion_drift",
            "sqlite_crash_recovery_ci_dry_run_docs_marker_parity_drift",
            "sqlite_crash_recovery_ci_dry_run_docs_remediation_marker_missing",
        ]
    )
    normalized_reason_codes = normalize_reason_codes(raw_reason_codes, ordered_reason_codes)

    status = "pass" if not normalized_reason_codes else "fail"
    final_decision = "GO" if not normalized_reason_codes else "NO-GO"
    reason_value = reason_codes_value(normalized_reason_codes)
    contract_status = "verified" if status == "pass" else "violation"

    report_payload = {
        "schema_version": policy_schema_version,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": reason_taxonomy_version,
        "reason_codes_csv": reason_codes_csv,
        "reason_codes": normalized_reason_codes,
        "reason_codes_value": reason_value,
        "sqlite_crash_recovery_ci_dry_run_contract_status": contract_status,
        "sqlite_crash_recovery_ci_dry_run_threshold_status": threshold_status,
        "sqlite_crash_recovery_ci_dry_run_reports_status": reports_status,
        "sqlite_crash_recovery_ci_dry_run_selector_status": selector_status,
        "sqlite_crash_recovery_ci_dry_run_workflow_status": workflow_status,
        "sqlite_crash_recovery_ci_dry_run_docs_status": docs_status,
        "sqlite_crash_recovery_ci_dry_run_docs_remediation_status": docs_remediation_status,
        "sqlite_crash_recovery_ci_dry_run_max_seconds": thresholds.get(
            "SQLITE_CRASH_RECOVERY_CI_DRY_RUN_MAX_SECONDS",
            "unknown",
        ),
        "inputs": {
            "sqlite_crash_recovery_summary_report_file": str(
                args.sqlite_crash_recovery_summary_report_file
            ),
            "sqlite_crash_recovery_policy_report_file": str(
                args.sqlite_crash_recovery_policy_report_file
            ),
            "sqlite_crash_recovery_contract_lane_report_file": str(
                args.sqlite_crash_recovery_contract_lane_report_file
            ),
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
            json.dumps(report_payload, sort_keys=True, indent=2) + "\n",
            encoding="utf-8",
        )

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={reason_taxonomy_version}")
    print(f"reason_codes_csv={reason_codes_csv}")
    print(f"reason_codes_value={reason_value}")
    print(f"sqlite_crash_recovery_ci_dry_run_contract_status={contract_status}")
    print(f"sqlite_crash_recovery_ci_dry_run_threshold_status={threshold_status}")
    print(f"sqlite_crash_recovery_ci_dry_run_reports_status={reports_status}")
    print(f"sqlite_crash_recovery_ci_dry_run_selector_status={selector_status}")
    print(f"sqlite_crash_recovery_ci_dry_run_workflow_status={workflow_status}")
    print(f"sqlite_crash_recovery_ci_dry_run_docs_status={docs_status}")
    print(
        "sqlite_crash_recovery_ci_dry_run_docs_remediation_status="
        f"{docs_remediation_status}"
    )
    print(
        "sqlite_crash_recovery_ci_dry_run_max_seconds="
        f"{thresholds.get('SQLITE_CRASH_RECOVERY_CI_DRY_RUN_MAX_SECONDS', 'unknown')}"
    )

    return 0 if status == "pass" else 1


if __name__ == "__main__":
    raise SystemExit(main())
