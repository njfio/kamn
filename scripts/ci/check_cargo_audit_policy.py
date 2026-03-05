#!/usr/bin/env python3
"""Fail-closed cargo-audit policy checker for CI gates."""

from __future__ import annotations

import argparse
import json
import re
import time
from datetime import date
from pathlib import Path
from typing import Any

SCHEMA_VERSION = "kamn.ci.cargo-audit-policy-report.v1"
WAIVER_SCHEMA_VERSION = "kamn.ci.cargo-audit-waiver.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.cargo-audit-policy-reason-taxonomy.v1"
ORDERED_REASON_CODES = (
    "cargo_audit_report_missing",
    "cargo_audit_report_invalid",
    "cargo_audit_report_schema_invalid",
    "cargo_audit_threshold_invalid",
    "cargo_audit_waiver_file_missing",
    "cargo_audit_waiver_invalid",
    "cargo_audit_waiver_schema_invalid",
    "cargo_audit_waiver_tracking_issue_invalid",
    "cargo_audit_waiver_expired",
    "cargo_audit_advisory_id_missing",
    "cargo_audit_advisory_severity_unknown",
    "cargo_audit_advisory_threshold_exceeded_unwaived",
    "cargo_audit_advisory_threshold_exceeded_waived",
)
SEVERITY_ORDER = {"low": 0, "moderate": 1, "high": 2, "critical": 3}
SEVERITY_ALIASES = {
    "low": "low",
    "moderate": "moderate",
    "medium": "moderate",
    "high": "high",
    "critical": "critical",
}
ISSUE_RE = re.compile(r"^#\d+$")
SCORE_RE = re.compile(r"([0-9]+(?:\.[0-9]+)?)")


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--audit-json", default="cargo-audit-report.json")
    parser.add_argument("--waiver-file", default=".ci/cargo-audit-waivers.json")
    parser.add_argument("--threshold-max-severity", default="moderate")
    parser.add_argument("--as-of-date", default=date.today().isoformat())
    parser.add_argument("--output-json", default="")
    return parser.parse_args()


def normalize_severity(value: Any) -> str | None:
    if not isinstance(value, str):
        return None
    return SEVERITY_ALIASES.get(value.strip().lower())


def score_to_severity(score: float) -> str:
    if score >= 9.0:
        return "critical"
    if score >= 7.0:
        return "high"
    if score >= 4.0:
        return "moderate"
    return "low"


def extract_score(value: Any) -> float | None:
    if isinstance(value, (int, float)):
        return float(value)
    if isinstance(value, str):
        match = SCORE_RE.search(value)
        return float(match.group(1)) if match else None
    if isinstance(value, dict):
        for key in ("score", "base_score", "baseScore"):
            score = extract_score(value.get(key))
            if score is not None:
                return score
    return None


def load_json(path: Path, reason_codes: set[str], violations: list[str], missing_code: str, invalid_code: str) -> Any:
    if not path.exists():
        reason_codes.add(missing_code)
        violations.append(f"missing file: {path}")
        return None
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as error:
        reason_codes.add(invalid_code)
        violations.append(f"invalid JSON {path}: {error}")
        return None


def advisory_rows(payload: Any) -> list[Any] | None:
    if not isinstance(payload, dict):
        return None
    for key in ("vulnerabilities", "advisories"):
        container = payload.get(key)
        if isinstance(container, list):
            return container
        if isinstance(container, dict):
            rows = container.get("list")
            if isinstance(rows, list):
                return rows
            if rows is None and (container.get("count") == 0 or container.get("found") is False):
                return []
    return None


def advisory_id(row: Any) -> str:
    if not isinstance(row, dict):
        return ""
    advisory = row.get("advisory")
    if isinstance(advisory, dict) and isinstance(advisory.get("id"), str):
        return advisory["id"].strip()
    if isinstance(row.get("id"), str):
        return row["id"].strip()
    return ""


def advisory_package(row: Any) -> str:
    if not isinstance(row, dict):
        return ""
    package = row.get("package")
    if isinstance(package, dict) and isinstance(package.get("name"), str):
        return package["name"].strip()
    if isinstance(package, str):
        return package.strip()
    return ""


def advisory_severity(row: Any) -> str | None:
    if not isinstance(row, dict):
        return None
    advisory = row.get("advisory")
    if isinstance(advisory, dict):
        direct = normalize_severity(advisory.get("severity"))
        if direct is not None:
            return direct
    direct = normalize_severity(row.get("severity"))
    if direct is not None:
        return direct
    cvss = advisory.get("cvss") if isinstance(advisory, dict) else row.get("cvss")
    score = extract_score(cvss)
    return score_to_severity(score) if score is not None else None


def parse_waivers(
    payload: Any,
    as_of: date,
    reason_codes: set[str],
    violations: list[str],
) -> dict[str, list[dict[str, str]]]:
    if payload is None:
        return {}
    if not isinstance(payload, dict):
        reason_codes.add("cargo_audit_waiver_schema_invalid")
        violations.append("waiver payload must be an object")
        return {}
    if payload.get("schema_version") != WAIVER_SCHEMA_VERSION:
        reason_codes.add("cargo_audit_waiver_schema_invalid")
        violations.append("waiver schema_version mismatch")
        return {}
    rows = payload.get("waivers")
    if not isinstance(rows, list):
        reason_codes.add("cargo_audit_waiver_schema_invalid")
        violations.append("waiver payload must include waivers[]")
        return {}

    waivers: dict[str, list[dict[str, str]]] = {}
    for idx, row in enumerate(rows):
        if not isinstance(row, dict):
            reason_codes.add("cargo_audit_waiver_invalid")
            violations.append(f"waiver[{idx}] must be object")
            continue
        aid = row.get("advisory_id")
        reason = row.get("reason")
        issue = row.get("tracking_issue")
        expires = row.get("expires_on")
        package = row.get("package")
        if not all(isinstance(v, str) and v.strip() for v in (aid, reason, issue, expires)):
            reason_codes.add("cargo_audit_waiver_invalid")
            violations.append(f"waiver[{idx}] requires advisory_id/reason/tracking_issue/expires_on")
            continue
        if ISSUE_RE.fullmatch(issue.strip()) is None:
            reason_codes.add("cargo_audit_waiver_tracking_issue_invalid")
            violations.append(f"waiver[{idx}] tracking_issue must match #<issue-id>")
            continue
        try:
            expiry = date.fromisoformat(expires.strip())
        except ValueError:
            reason_codes.add("cargo_audit_waiver_invalid")
            violations.append(f"waiver[{idx}] expires_on must be YYYY-MM-DD")
            continue
        if expiry < as_of:
            reason_codes.add("cargo_audit_waiver_expired")
            violations.append(f"waiver[{idx}] expired on {expiry.isoformat()}")
            continue
        if package is not None and (not isinstance(package, str) or not package.strip()):
            reason_codes.add("cargo_audit_waiver_invalid")
            violations.append(f"waiver[{idx}] package must be omitted or non-empty")
            continue
        waivers.setdefault(aid.strip(), []).append(
            {"package": package.strip() if isinstance(package, str) else ""}
        )
    return waivers


def ordered_reason_codes(reason_codes: set[str]) -> list[str]:
    order = {code: idx for idx, code in enumerate(ORDERED_REASON_CODES)}
    return sorted(reason_codes, key=lambda code: (order.get(code, len(order)), code))


def elapsed_seconds(started_at: float) -> float:
    return round(time.perf_counter() - started_at, 6)


def main() -> int:
    started_at = time.perf_counter()
    args = parse_args()
    reason_codes: set[str] = set()
    violations: list[str] = []

    try:
        as_of = date.fromisoformat(args.as_of_date)
    except ValueError:
        as_of = date.today()
        reason_codes.add("cargo_audit_waiver_invalid")
        violations.append("as-of-date must be YYYY-MM-DD")

    threshold = normalize_severity(args.threshold_max_severity)
    if threshold is None:
        threshold = "moderate"
        reason_codes.add("cargo_audit_threshold_invalid")
        violations.append("threshold-max-severity must be low|moderate|high|critical")
    threshold_rank = SEVERITY_ORDER[threshold]

    audit_payload = load_json(
        Path(args.audit_json),
        reason_codes,
        violations,
        "cargo_audit_report_missing",
        "cargo_audit_report_invalid",
    )
    rows = advisory_rows(audit_payload) if audit_payload is not None else []
    if rows is None:
        rows = []
        reason_codes.add("cargo_audit_report_schema_invalid")
        violations.append("unsupported cargo-audit report schema")

    waiver_payload = load_json(
        Path(args.waiver_file),
        reason_codes,
        violations,
        "cargo_audit_waiver_file_missing",
        "cargo_audit_waiver_invalid",
    )
    waivers = parse_waivers(waiver_payload, as_of, reason_codes, violations)

    advisory_total = 0
    unknown_total = 0
    exceeded_total = 0
    unwaived_total = 0
    waived_total = 0
    unwaived_markers: list[str] = []
    unknown_markers: list[str] = []

    for idx, row in enumerate(rows):
        aid = advisory_id(row)
        if not aid:
            reason_codes.add("cargo_audit_advisory_id_missing")
            violations.append(f"advisory[{idx}] missing id")
            continue
        advisory_total += 1
        package = advisory_package(row)
        severity = advisory_severity(row)
        if severity is None:
            unknown_total += 1
            reason_codes.add("cargo_audit_advisory_severity_unknown")
            unknown_markers.append(f"{aid}:{package}")
            continue
        if SEVERITY_ORDER[severity] <= threshold_rank:
            continue
        exceeded_total += 1
        candidates = waivers.get(aid, [])
        matched = any(not candidate["package"] or candidate["package"] == package for candidate in candidates)
        if matched:
            waived_total += 1
        else:
            unwaived_total += 1
            reason_codes.add("cargo_audit_advisory_threshold_exceeded_unwaived")
            unwaived_markers.append(f"{aid}:{package}:{severity}")

    review_required = waived_total > 0
    if review_required and unwaived_total == 0:
        reason_codes.add("cargo_audit_advisory_threshold_exceeded_waived")

    status = "fail" if violations or unknown_total > 0 or unwaived_total > 0 else "pass"
    ordered = ordered_reason_codes(reason_codes)
    reason_codes_csv = ",".join(ordered) if ordered else "none"
    reason_class = "violation" if status == "fail" else ("waived" if review_required else "stable")
    policy_elapsed_seconds = elapsed_seconds(started_at)

    report = {
        "schema_version": SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "status": status,
        "reason_class": reason_class,
        "reason_codes": ordered if ordered else ["none"],
        "reason_codes_csv": reason_codes_csv,
        "reason_codes_value": reason_codes_csv,
        "threshold_max_severity": threshold,
        "audit_report_file": args.audit_json,
        "waiver_file": args.waiver_file,
        "as_of_date": as_of.isoformat(),
        "advisory_total": advisory_total,
        "threshold_exceeded_total": exceeded_total,
        "waived_total": waived_total,
        "unwaived_total": unwaived_total,
        "unknown_severity_total": unknown_total,
        "review_required": review_required,
        "policy_elapsed_seconds": policy_elapsed_seconds,
        "violations": violations,
    }
    if args.output_json:
        output = Path(args.output_json)
        output.parent.mkdir(parents=True, exist_ok=True)
        output.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print("status=ok" if status == "pass" else "status=fail")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes_csv={reason_codes_csv}")
    print(f"reason_codes_value={reason_codes_csv}")
    print(f"reason_class={reason_class}")
    print(f"threshold_max_severity={threshold}")
    print(f"advisory_total={advisory_total}")
    print(f"threshold_exceeded_total={exceeded_total}")
    print(f"waived_total={waived_total}")
    print(f"unwaived_total={unwaived_total}")
    print(f"unknown_severity_total={unknown_total}")
    print(f"review_required={'true' if review_required else 'false'}")
    print(f"policy_elapsed_seconds={policy_elapsed_seconds}")
    if status == "fail":
        for violation in violations:
            print(f"violation={violation}")
        for marker in unknown_markers:
            print(f"unknown_severity_advisory={marker}")
        for marker in unwaived_markers:
            print(f"unwaived_threshold_exceeded_advisory={marker}")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
