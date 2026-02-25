#!/usr/bin/env python3

import argparse
import json
import subprocess
from datetime import date
from pathlib import Path

REASON_TAXONOMY_VERSION = "kamn.ci.combined-shell-surface-trend-policy-reason-taxonomy.v1"
REASON_CODES_CSV = ",".join(
    [
        "combined_shell_surface_budget_status_fail",
        "combined_shell_surface_decline_window_fail_exceeded",
        "combined_shell_surface_decline_window_warn_exceeded",
        "combined_shell_surface_delta_ratio_invalid",
        "combined_shell_surface_delta_script_count_invalid",
        "combined_shell_surface_delta_shell_line_total_invalid",
        "combined_shell_surface_ratio_delta_fail_exceeded",
        "combined_shell_surface_ratio_delta_warn_exceeded",
        "combined_shell_surface_ratio_fail_ceiling_exceeded",
        "combined_shell_surface_ratio_invalid",
        "combined_shell_surface_ratio_warn_ceiling_exceeded",
        "combined_shell_surface_report_schema_mismatch",
        "combined_shell_surface_rust_line_total_invalid",
        "combined_shell_surface_script_count_delta_fail_exceeded",
        "combined_shell_surface_script_count_delta_warn_exceeded",
        "combined_shell_surface_script_count_invalid",
        "combined_shell_surface_shell_line_total_delta_fail_exceeded",
        "combined_shell_surface_shell_line_total_delta_warn_exceeded",
        "combined_shell_surface_shell_line_total_invalid",
        "combined_shell_surface_threshold_date_invalid",
        "combined_shell_surface_threshold_file_stale",
        "combined_shell_surface_threshold_order_invalid",
        "combined_shell_surface_threshold_schema_mismatch",
        "combined_shell_surface_threshold_value_invalid",
        "combined_shell_surface_today_override_invalid",
        "combined_shell_surface_governance_commit_count_invalid",
        "combined_shell_surface_governance_mitigation_issue_missing",
        "combined_shell_surface_governance_non_merge_commit_count_invalid",
        "combined_shell_surface_governance_ratio_fail_exceeded",
        "combined_shell_surface_governance_ratio_invalid",
        "combined_shell_surface_governance_ratio_mismatch",
        "combined_shell_surface_governance_ratio_warn_reduction_contract_active",
        "combined_shell_surface_governance_release_invalid",
        "combined_shell_surface_governance_section_missing",
        "combined_shell_surface_governance_status_invalid",
        "combined_shell_surface_governance_target_ratio_invalid",
        "combined_shell_surface_governance_runtime_test_ratio_fail_exceeded",
        "combined_shell_surface_governance_runtime_test_ratio_invalid",
        "combined_shell_surface_governance_runtime_test_ratio_warn_exceeded",
        "combined_shell_surface_governance_runtime_test_section_missing",
        "combined_shell_surface_governance_test_line_total_invalid",
        "combined_shell_surface_runtime_test_line_total_invalid",
    ]
)


def build_parser(root_dir: Path) -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report-file", default="")
    parser.add_argument(
        "--threshold-file",
        default=str(root_dir / "fixtures" / "ci" / "combined_shell_surface_trend_thresholds.json"),
    )
    parser.add_argument("--output-json", default="")
    parser.add_argument("--today", default="")
    return parser


def add_reason(reason_codes: list[str], code: str) -> None:
    if code not in reason_codes:
        reason_codes.append(code)


def main() -> int:
    root_dir = Path(__file__).resolve().parents[2]
    args = build_parser(root_dir).parse_args()

    report_file = Path(args.report_file).resolve() if args.report_file else None
    threshold_file = Path(args.threshold_file).resolve()
    output_json = Path(args.output_json).resolve() if args.output_json else None
    today_override = args.today.strip()

    tmp_dir = None
    if report_file is None:
        import tempfile

        tmp_dir = tempfile.TemporaryDirectory()
        report_file = Path(tmp_dir.name) / "combined-shell-surface-trend-report.json"
        generator = root_dir / "scripts" / "ci" / "generate_combined_shell_surface_trend_report.sh"
        subprocess.run(
            ["bash", str(generator), "--output-json", str(report_file)],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
            check=True,
        )

    if not report_file.is_file():
        raise SystemExit(f"report file not found: {report_file}")
    if not threshold_file.is_file():
        raise SystemExit(f"threshold file not found: {threshold_file}")
    if output_json is None:
        if tmp_dir is None:
            import tempfile

            tmp_dir = tempfile.TemporaryDirectory()
        output_json = Path(tmp_dir.name) / "combined-shell-surface-trend-policy-report.json"
    output_json.parent.mkdir(parents=True, exist_ok=True)

    report = json.loads(report_file.read_text(encoding="utf-8"))
    thresholds = json.loads(threshold_file.read_text(encoding="utf-8"))

    reason_codes: list[str] = []

    if report.get("schema_version") != "kamn.ci.combined-shell-surface-trend-report.v1":
        add_reason(reason_codes, "combined_shell_surface_report_schema_mismatch")
    if thresholds.get("schema_version") != "kamn.ci.combined-shell-surface-trend-thresholds.v1":
        add_reason(reason_codes, "combined_shell_surface_threshold_schema_mismatch")

    current = report.get("current", {})
    deltas = report.get("deltas", {})
    script_budget = report.get("script_budget", {})
    governance = report.get("governance_structural_coupling", {})
    governance_runtime_test_ratio = report.get("governance_runtime_test_ratio", {})

    script_count = current.get("script_count")
    shell_line_total = current.get("shell_line_total")
    rust_line_total = current.get("rust_line_total")
    shell_to_rust_ratio = current.get("shell_to_rust_ratio")
    delta_script_count = deltas.get("script_count")
    delta_shell_line_total = deltas.get("shell_line_total")
    delta_shell_to_rust_ratio = deltas.get("shell_to_rust_ratio")
    script_budget_status = script_budget.get("status")
    governance_release = governance.get("release")
    governance_status = governance.get("status")
    governance_non_merge_commit_count = governance.get("non_merge_commit_count")
    governance_commit_count = governance.get("governance_commit_count")
    governance_commit_ratio = governance.get("governance_commit_ratio")
    governance_target_ratio_max = governance.get("target_ratio_max")
    governance_mitigation_issue_marker = str(governance.get("mitigation_issue_marker", "none")).strip()
    governance_runtime_test_ratio_status = governance_runtime_test_ratio.get("status")
    governance_test_line_total = governance_runtime_test_ratio.get("governance_test_line_total")
    runtime_test_line_total = governance_runtime_test_ratio.get("runtime_test_line_total")
    governance_test_ratio = governance_runtime_test_ratio.get("governance_test_ratio")

    if script_budget_status != "pass":
        add_reason(reason_codes, "combined_shell_surface_budget_status_fail")
    if not isinstance(governance, dict) or not governance:
        add_reason(reason_codes, "combined_shell_surface_governance_section_missing")
    if (
        not isinstance(governance_runtime_test_ratio, dict)
        or not governance_runtime_test_ratio
    ):
        add_reason(reason_codes, "combined_shell_surface_governance_runtime_test_section_missing")

    warn_codes: list[str] = []
    fail_codes: list[str] = []

    warn_script_count_increase = 0
    fail_script_count_increase = 0
    warn_shell_line_total_increase = 0
    fail_shell_line_total_increase = 0
    warn_shell_to_rust_ratio = 0.0
    fail_shell_to_rust_ratio = 0.0
    warn_shell_to_rust_ratio_increase = 0.0
    fail_shell_to_rust_ratio_increase = 0.0
    warn_non_declining_window_days = 0
    fail_non_declining_window_days = 0
    warn_governance_runtime_test_ratio = 0.0
    fail_governance_runtime_test_ratio = 0.0
    threshold_max_age_days = 0
    threshold_refreshed_on = ""

    try:
        warn_script_count_increase = int(thresholds["warn_script_count_increase"])
        fail_script_count_increase = int(thresholds["fail_script_count_increase"])
        warn_shell_line_total_increase = int(thresholds["warn_shell_line_total_increase"])
        fail_shell_line_total_increase = int(thresholds["fail_shell_line_total_increase"])
        warn_shell_to_rust_ratio = float(thresholds["warn_shell_to_rust_ratio"])
        fail_shell_to_rust_ratio = float(thresholds["fail_shell_to_rust_ratio"])
        warn_shell_to_rust_ratio_increase = float(thresholds["warn_shell_to_rust_ratio_increase"])
        fail_shell_to_rust_ratio_increase = float(thresholds["fail_shell_to_rust_ratio_increase"])
        warn_non_declining_window_days = int(thresholds["warn_non_declining_window_days"])
        fail_non_declining_window_days = int(thresholds["fail_non_declining_window_days"])
        warn_governance_runtime_test_ratio = float(thresholds["warn_governance_runtime_test_ratio"])
        fail_governance_runtime_test_ratio = float(thresholds["fail_governance_runtime_test_ratio"])
        threshold_max_age_days = int(thresholds["threshold_max_age_days"])
        threshold_refreshed_on = str(thresholds["threshold_refreshed_on"])
    except Exception:
        add_reason(reason_codes, "combined_shell_surface_threshold_value_invalid")

    if warn_script_count_increase >= fail_script_count_increase:
        add_reason(reason_codes, "combined_shell_surface_threshold_order_invalid")
    if warn_shell_line_total_increase >= fail_shell_line_total_increase:
        add_reason(reason_codes, "combined_shell_surface_threshold_order_invalid")
    if warn_shell_to_rust_ratio >= fail_shell_to_rust_ratio:
        add_reason(reason_codes, "combined_shell_surface_threshold_order_invalid")
    if warn_shell_to_rust_ratio_increase >= fail_shell_to_rust_ratio_increase:
        add_reason(reason_codes, "combined_shell_surface_threshold_order_invalid")
    if warn_non_declining_window_days >= fail_non_declining_window_days:
        add_reason(reason_codes, "combined_shell_surface_threshold_order_invalid")
    if warn_governance_runtime_test_ratio >= fail_governance_runtime_test_ratio:
        add_reason(reason_codes, "combined_shell_surface_threshold_order_invalid")
    if warn_governance_runtime_test_ratio <= 0 or fail_governance_runtime_test_ratio <= 0:
        add_reason(reason_codes, "combined_shell_surface_threshold_value_invalid")
    if warn_governance_runtime_test_ratio >= 1 or fail_governance_runtime_test_ratio >= 1:
        add_reason(reason_codes, "combined_shell_surface_threshold_value_invalid")

    if not isinstance(script_count, int):
        add_reason(reason_codes, "combined_shell_surface_script_count_invalid")
    if not isinstance(shell_line_total, int):
        add_reason(reason_codes, "combined_shell_surface_shell_line_total_invalid")
    if not isinstance(rust_line_total, int) or rust_line_total <= 0:
        add_reason(reason_codes, "combined_shell_surface_rust_line_total_invalid")
    if not isinstance(shell_to_rust_ratio, (int, float)):
        add_reason(reason_codes, "combined_shell_surface_ratio_invalid")
    if not isinstance(delta_script_count, int):
        add_reason(reason_codes, "combined_shell_surface_delta_script_count_invalid")
    if not isinstance(delta_shell_line_total, int):
        add_reason(reason_codes, "combined_shell_surface_delta_shell_line_total_invalid")
    if not isinstance(delta_shell_to_rust_ratio, (int, float)):
        add_reason(reason_codes, "combined_shell_surface_delta_ratio_invalid")
    if not isinstance(governance_release, int) or governance_release <= 0:
        add_reason(reason_codes, "combined_shell_surface_governance_release_invalid")
    if not isinstance(governance_non_merge_commit_count, int) or governance_non_merge_commit_count <= 0:
        add_reason(reason_codes, "combined_shell_surface_governance_non_merge_commit_count_invalid")
    if (
        not isinstance(governance_commit_count, int)
        or governance_commit_count < 0
        or (
            isinstance(governance_non_merge_commit_count, int)
            and governance_non_merge_commit_count > 0
            and governance_commit_count > governance_non_merge_commit_count
        )
    ):
        add_reason(reason_codes, "combined_shell_surface_governance_commit_count_invalid")
    if not isinstance(governance_commit_ratio, (int, float)):
        add_reason(reason_codes, "combined_shell_surface_governance_ratio_invalid")
    if (
        not isinstance(governance_target_ratio_max, (int, float))
        or governance_target_ratio_max <= 0
        or governance_target_ratio_max >= 1
    ):
        add_reason(reason_codes, "combined_shell_surface_governance_target_ratio_invalid")
    if governance_status not in {"within_target", "reduction_contract_active", "over_target_unmitigated"}:
        add_reason(reason_codes, "combined_shell_surface_governance_status_invalid")
    if governance_runtime_test_ratio_status not in {"computed"}:
        add_reason(reason_codes, "combined_shell_surface_governance_runtime_test_ratio_invalid")
    if not isinstance(governance_test_line_total, int) or governance_test_line_total < 0:
        add_reason(reason_codes, "combined_shell_surface_governance_test_line_total_invalid")
    if not isinstance(runtime_test_line_total, int) or runtime_test_line_total < 0:
        add_reason(reason_codes, "combined_shell_surface_runtime_test_line_total_invalid")
    if not isinstance(governance_test_ratio, (int, float)):
        add_reason(reason_codes, "combined_shell_surface_governance_runtime_test_ratio_invalid")

    if (
        isinstance(governance_non_merge_commit_count, int)
        and governance_non_merge_commit_count > 0
        and isinstance(governance_commit_count, int)
        and isinstance(governance_commit_ratio, (int, float))
    ):
        computed_governance_ratio = round(
            governance_commit_count / governance_non_merge_commit_count,
            4,
        )
        if abs(float(governance_commit_ratio) - computed_governance_ratio) > 0.001:
            add_reason(reason_codes, "combined_shell_surface_governance_ratio_mismatch")
    else:
        computed_governance_ratio = None

    if (
        isinstance(governance_test_line_total, int)
        and governance_test_line_total >= 0
        and isinstance(runtime_test_line_total, int)
        and runtime_test_line_total >= 0
        and isinstance(governance_test_ratio, (int, float))
    ):
        total_governance_runtime_test_lines = governance_test_line_total + runtime_test_line_total
        if total_governance_runtime_test_lines <= 0:
            add_reason(reason_codes, "combined_shell_surface_runtime_test_line_total_invalid")
            computed_governance_runtime_test_ratio = None
        else:
            computed_governance_runtime_test_ratio = round(
                governance_test_line_total / total_governance_runtime_test_lines,
                4,
            )
            if (
                float(governance_test_ratio) < 0
                or float(governance_test_ratio) > 1
                or abs(float(governance_test_ratio) - computed_governance_runtime_test_ratio) > 0.001
            ):
                add_reason(reason_codes, "combined_shell_surface_governance_runtime_test_ratio_invalid")
    else:
        computed_governance_runtime_test_ratio = None

    if today_override:
        try:
            today = date.fromisoformat(today_override)
        except ValueError:
            add_reason(reason_codes, "combined_shell_surface_today_override_invalid")
            today = None
    else:
        today = date.today()

    threshold_date = None
    if threshold_refreshed_on:
        try:
            threshold_date = date.fromisoformat(threshold_refreshed_on)
        except ValueError:
            add_reason(reason_codes, "combined_shell_surface_threshold_date_invalid")

    threshold_age_days = None
    if threshold_date is not None and today is not None:
        threshold_age_days = (today - threshold_date).days
        if threshold_age_days < 0:
            add_reason(reason_codes, "combined_shell_surface_threshold_date_invalid")

    if threshold_age_days is not None and threshold_age_days > threshold_max_age_days:
        add_reason(reason_codes, "combined_shell_surface_threshold_file_stale")

    if not reason_codes:
        if delta_script_count > fail_script_count_increase:
            fail_codes.append("combined_shell_surface_script_count_delta_fail_exceeded")
        elif delta_script_count > warn_script_count_increase:
            warn_codes.append("combined_shell_surface_script_count_delta_warn_exceeded")

        if delta_shell_line_total > fail_shell_line_total_increase:
            fail_codes.append("combined_shell_surface_shell_line_total_delta_fail_exceeded")
        elif delta_shell_line_total > warn_shell_line_total_increase:
            warn_codes.append("combined_shell_surface_shell_line_total_delta_warn_exceeded")

        if shell_to_rust_ratio > fail_shell_to_rust_ratio:
            fail_codes.append("combined_shell_surface_ratio_fail_ceiling_exceeded")
        elif shell_to_rust_ratio > warn_shell_to_rust_ratio:
            warn_codes.append("combined_shell_surface_ratio_warn_ceiling_exceeded")

        if delta_shell_to_rust_ratio > fail_shell_to_rust_ratio_increase:
            fail_codes.append("combined_shell_surface_ratio_delta_fail_exceeded")
        elif delta_shell_to_rust_ratio > warn_shell_to_rust_ratio_increase:
            warn_codes.append("combined_shell_surface_ratio_delta_warn_exceeded")

        non_declining = (
            delta_script_count > 0
            or delta_shell_line_total > 0
            or delta_shell_to_rust_ratio > 0
        )
        if non_declining and threshold_age_days is not None:
            if threshold_age_days > fail_non_declining_window_days:
                fail_codes.append("combined_shell_surface_decline_window_fail_exceeded")
            elif threshold_age_days > warn_non_declining_window_days:
                warn_codes.append("combined_shell_surface_decline_window_warn_exceeded")

        if governance_commit_ratio > governance_target_ratio_max + 0.001:
            if governance_status == "reduction_contract_active":
                if governance_mitigation_issue_marker in {"", "none", "unknown"}:
                    fail_codes.append("combined_shell_surface_governance_mitigation_issue_missing")
                else:
                    warn_codes.append(
                        "combined_shell_surface_governance_ratio_warn_reduction_contract_active"
                    )
            else:
                fail_codes.append("combined_shell_surface_governance_ratio_fail_exceeded")

        if governance_test_ratio > fail_governance_runtime_test_ratio:
            fail_codes.append("combined_shell_surface_governance_runtime_test_ratio_fail_exceeded")
        elif governance_test_ratio > warn_governance_runtime_test_ratio:
            warn_codes.append("combined_shell_surface_governance_runtime_test_ratio_warn_exceeded")

    all_reason_codes = reason_codes + fail_codes + warn_codes

    if reason_codes or fail_codes:
        trend_status = "fail"
        policy_decision = "NO-GO"
        status = "fail"
    elif warn_codes:
        trend_status = "warn"
        policy_decision = "WARN"
        status = "ok"
    else:
        trend_status = "within"
        policy_decision = "GO"
        status = "ok"

    if fail_codes:
        remediation = "Prioritize wrapper-family consolidation and shared dispatch extraction in non-Kolme lanes."
    elif warn_codes:
        remediation = "Monitor shell-surface growth and schedule next tranche before fail threshold breach."
    else:
        remediation = "none"

    policy_report = {
        "schema_version": "kamn.ci.combined-shell-surface-trend-policy-report.v1",
        "status": status,
        "policy_decision": policy_decision,
        "trend_status": trend_status,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": REASON_CODES_CSV,
        "reason_codes": all_reason_codes,
        "reason_codes_value": "none" if not all_reason_codes else ",".join(all_reason_codes),
        "remediation": remediation,
        "current": {
            "script_count": script_count,
            "shell_line_total": shell_line_total,
            "rust_line_total": rust_line_total,
            "shell_to_rust_ratio": shell_to_rust_ratio,
        },
        "deltas": {
            "script_count": delta_script_count,
            "shell_line_total": delta_shell_line_total,
            "shell_to_rust_ratio": delta_shell_to_rust_ratio,
        },
        "thresholds": {
            "warn_script_count_increase": warn_script_count_increase,
            "fail_script_count_increase": fail_script_count_increase,
            "warn_shell_line_total_increase": warn_shell_line_total_increase,
            "fail_shell_line_total_increase": fail_shell_line_total_increase,
            "warn_shell_to_rust_ratio": warn_shell_to_rust_ratio,
            "fail_shell_to_rust_ratio": fail_shell_to_rust_ratio,
            "warn_shell_to_rust_ratio_increase": warn_shell_to_rust_ratio_increase,
            "fail_shell_to_rust_ratio_increase": fail_shell_to_rust_ratio_increase,
            "threshold_refreshed_on": threshold_refreshed_on,
            "threshold_max_age_days": threshold_max_age_days,
            "warn_non_declining_window_days": warn_non_declining_window_days,
            "fail_non_declining_window_days": fail_non_declining_window_days,
            "warn_governance_runtime_test_ratio": warn_governance_runtime_test_ratio,
            "fail_governance_runtime_test_ratio": fail_governance_runtime_test_ratio,
            "threshold_age_days": threshold_age_days,
        },
        "governance_structural_coupling_status": governance_status,
        "governance_structural_coupling": {
            "release": governance_release,
            "status": governance_status,
            "non_merge_commit_count": governance_non_merge_commit_count,
            "governance_commit_count": governance_commit_count,
            "governance_commit_ratio": governance_commit_ratio,
            "computed_governance_commit_ratio": computed_governance_ratio,
            "target_ratio_max": governance_target_ratio_max,
            "mitigation_issue_marker": governance_mitigation_issue_marker,
        },
        "governance_runtime_test_ratio": {
            "status": governance_runtime_test_ratio_status,
            "governance_test_line_total": governance_test_line_total,
            "runtime_test_line_total": runtime_test_line_total,
            "governance_test_ratio": governance_test_ratio,
            "computed_governance_test_ratio": computed_governance_runtime_test_ratio,
            "warn_governance_runtime_test_ratio": warn_governance_runtime_test_ratio,
            "fail_governance_runtime_test_ratio": fail_governance_runtime_test_ratio,
        },
    }

    output_json.write_text(json.dumps(policy_report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    reason_marker = "none" if not all_reason_codes else ",".join(all_reason_codes)
    print(f"status={status}")
    print(f"policy_decision={policy_decision}")
    print(f"trend_status={trend_status}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"reason_codes={reason_marker}")
    print(f"reason_codes_value={reason_marker}")
    print(f"governance_structural_coupling_status={governance_status}")
    print(f"script_count={script_count}")
    print(f"shell_line_total={shell_line_total}")
    print(f"rust_line_total={rust_line_total}")
    print(f"shell_to_rust_ratio={shell_to_rust_ratio}")
    print(f"delta_script_count={delta_script_count}")
    print(f"delta_shell_line_total={delta_shell_line_total}")
    print(f"delta_shell_to_rust_ratio={delta_shell_to_rust_ratio}")
    print(f"governance_runtime_test_ratio={governance_test_ratio}")
    print(f"governance_test_line_total={governance_test_line_total}")
    print(f"runtime_test_line_total={runtime_test_line_total}")
    print(f"remediation={remediation}")

    if tmp_dir is not None:
        tmp_dir.cleanup()
    return 0 if status == "ok" else 1


if __name__ == "__main__":
    raise SystemExit(main())
