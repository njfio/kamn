#!/usr/bin/env python3
"""Fail-closed stale-script reference detector for deleted/superseded entrypoints."""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path
from typing import Any

DELETION_MANIFEST_SCHEMA_VERSION = "kamn.ci.superseded-script-deletion-manifest.v1"
CHECK_REPORT_SCHEMA_VERSION = "kamn.ci.stale-script-reference-detector-report.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.stale-script-reference-detector-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "stale_script_reference_argument_invalid,"
    "stale_script_reference_deletion_manifest_missing,"
    "stale_script_reference_deletion_manifest_schema_invalid,"
    "stale_script_reference_detected,"
    "stale_script_reference_manifest_entry_invalid,"
    "stale_script_reference_output_json_required,"
    "stale_script_reference_output_write_failed,"
    "stale_script_reference_scan_root_missing"
)
DEFAULT_DELETION_MANIFEST_FILE = "fixtures/ci/superseded_script_deletion_manifest.json"
DEFAULT_SCAN_ROOTS = ["docs", ".github/workflows", "scripts/framework/manifests"]


class CheckerError(RuntimeError):
    """Raised for deterministic checker failures."""


def fail(message: str) -> None:
    raise CheckerError(message)


def parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Detect stale references to scripts listed in superseded/deletion manifests."
        )
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root used to resolve relative paths.",
    )
    parser.add_argument(
        "--deletion-manifest-file",
        default=DEFAULT_DELETION_MANIFEST_FILE,
        help="Path to superseded-script deletion manifest JSON.",
    )
    parser.add_argument(
        "--scan-root",
        action="append",
        default=[],
        help=(
            "Relative root to scan for stale script references. "
            "Can be supplied multiple times."
        ),
    )
    parser.add_argument(
        "--output-json",
        required=True,
        help="Path to write stale-reference detector report JSON.",
    )
    return parser.parse_args(argv)


def resolve_path(*, repo_root: Path, value: str) -> Path:
    path = Path(value)
    if not path.is_absolute():
        path = (repo_root / path).resolve()
    return path


def write_json(path: Path, payload: dict[str, Any]) -> None:
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    except OSError as exc:
        fail(f"failed to write output report JSON: {path}: {exc}")


def to_repo_relative(path: Path, repo_root: Path) -> str:
    try:
        return path.resolve().relative_to(repo_root.resolve()).as_posix()
    except ValueError:
        return path.as_posix()


def load_json_object(path: Path, *, label: str) -> dict[str, Any]:
    if not path.is_file():
        fail(f"{label} not found: {path}")
    try:
        payload = json.loads(path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as exc:
        fail(f"{label} is not valid JSON: {path}: {exc}")
    if not isinstance(payload, dict):
        fail(f"{label} must be a JSON object: {path}")
    return payload


def parse_deleted_script_paths(manifest_payload: dict[str, Any]) -> tuple[list[str], list[str]]:
    reasons: list[str] = []
    if manifest_payload.get("schema_version") != DELETION_MANIFEST_SCHEMA_VERSION:
        return [], ["stale_script_reference_deletion_manifest_schema_invalid"]

    deletions = manifest_payload.get("deletions")
    if not isinstance(deletions, list):
        return [], ["stale_script_reference_deletion_manifest_schema_invalid"]

    deleted_script_paths: list[str] = []
    seen_paths: set[str] = set()
    for entry in deletions:
        if not isinstance(entry, dict):
            reasons.append("stale_script_reference_manifest_entry_invalid")
            continue
        script_path = entry.get("script_path")
        reason_code = entry.get("reason_code")
        if not isinstance(script_path, str) or not script_path.strip():
            reasons.append("stale_script_reference_manifest_entry_invalid")
            continue
        if not isinstance(reason_code, str) or not reason_code.strip():
            reasons.append("stale_script_reference_manifest_entry_invalid")
            continue
        normalized_script_path = script_path.strip()
        if normalized_script_path in seen_paths:
            reasons.append("stale_script_reference_manifest_entry_invalid")
            continue
        seen_paths.add(normalized_script_path)
        deleted_script_paths.append(normalized_script_path)
    return sorted(deleted_script_paths), sorted(set(reasons))


def filter_enforced_deleted_script_paths(
    *,
    repo_root: Path,
    deleted_script_paths: list[str],
) -> list[str]:
    enforced_paths: list[str] = []
    for deleted_script_path in deleted_script_paths:
        candidate_path = resolve_path(repo_root=repo_root, value=deleted_script_path)
        if candidate_path.exists():
            # Transitional state: entry is scheduled for a deletion wave but not yet removed.
            continue
        enforced_paths.append(deleted_script_path)
    return sorted(enforced_paths)


def collect_scan_files(*, repo_root: Path, scan_roots: list[str]) -> tuple[list[Path], list[str]]:
    files: list[Path] = []
    reasons: list[str] = []
    for scan_root in scan_roots:
        scan_root_path = resolve_path(repo_root=repo_root, value=scan_root)
        if not scan_root_path.exists():
            reasons.append("stale_script_reference_scan_root_missing")
            continue
        if scan_root_path.is_file():
            files.append(scan_root_path)
            continue
        if not scan_root_path.is_dir():
            reasons.append("stale_script_reference_scan_root_missing")
            continue
        for file_path in sorted(path for path in scan_root_path.rglob("*") if path.is_file()):
            files.append(file_path)
    deduped_files: dict[str, Path] = {path.resolve().as_posix(): path for path in files}
    return sorted(deduped_files.values(), key=lambda path: path.as_posix()), sorted(set(reasons))


def find_stale_references(
    *,
    repo_root: Path,
    deleted_script_paths: list[str],
    files: list[Path],
) -> list[dict[str, str]]:
    findings: list[dict[str, str]] = []
    for file_path in files:
        try:
            contents = file_path.read_text(encoding="utf-8", errors="ignore")
        except OSError:
            continue
        for deleted_script_path in deleted_script_paths:
            if deleted_script_path in contents:
                findings.append(
                    {
                        "script_path": deleted_script_path,
                        "reference_file": to_repo_relative(file_path, repo_root),
                    }
                )
    findings.sort(key=lambda item: (item["script_path"], item["reference_file"]))
    return findings


def main(argv: list[str]) -> int:
    try:
        args = parse_args(argv)
    except SystemExit:
        print("status=fail")
        print("final_decision=NO-GO")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print("reason_codes=stale_script_reference_argument_invalid")
        print(f"reason_codes_csv={REASON_CODES_CSV}")
        print("deletion_entry_count=0")
        print("enforced_deletion_entry_count=0")
        print("scan_root_count=0")
        print("scanned_file_count=0")
        print("stale_reference_count=0")
        return 1

    output_json = args.output_json.strip() if isinstance(args.output_json, str) else ""
    if not output_json:
        print("status=fail")
        print("final_decision=NO-GO")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print("reason_codes=stale_script_reference_output_json_required")
        print(f"reason_codes_csv={REASON_CODES_CSV}")
        print("deletion_entry_count=0")
        print("enforced_deletion_entry_count=0")
        print("scan_root_count=0")
        print("scanned_file_count=0")
        print("stale_reference_count=0")
        return 1

    repo_root = Path(args.repo_root).resolve()
    if not repo_root.is_dir():
        print("status=fail")
        print("final_decision=NO-GO")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print("reason_codes=stale_script_reference_argument_invalid")
        print(f"reason_codes_csv={REASON_CODES_CSV}")
        print("deletion_entry_count=0")
        print("enforced_deletion_entry_count=0")
        print("scan_root_count=0")
        print("scanned_file_count=0")
        print("stale_reference_count=0")
        print(f"error=repo root is not a directory: {repo_root}")
        return 1

    deletion_manifest_path = resolve_path(
        repo_root=repo_root,
        value=args.deletion_manifest_file,
    )
    output_json_path = resolve_path(repo_root=repo_root, value=output_json)
    scan_roots = args.scan_root or DEFAULT_SCAN_ROOTS

    reason_codes: list[str] = []
    findings: list[dict[str, str]] = []
    deleted_script_paths: list[str] = []
    enforced_deleted_script_paths: list[str] = []
    scan_files: list[Path] = []
    try:
        manifest_payload = load_json_object(
            deletion_manifest_path,
            label="deletion manifest file",
        )
        deleted_script_paths, manifest_reasons = parse_deleted_script_paths(manifest_payload)
        reason_codes.extend(manifest_reasons)
        enforced_deleted_script_paths = filter_enforced_deleted_script_paths(
            repo_root=repo_root,
            deleted_script_paths=deleted_script_paths,
        )

        scan_files, scan_reasons = collect_scan_files(repo_root=repo_root, scan_roots=scan_roots)
        reason_codes.extend(scan_reasons)

        if not reason_codes and enforced_deleted_script_paths:
            findings = find_stale_references(
                repo_root=repo_root,
                deleted_script_paths=enforced_deleted_script_paths,
                files=scan_files,
            )
            if findings:
                reason_codes.append("stale_script_reference_detected")
    except CheckerError as error:
        error_message = str(error)
        if "not found" in error_message and "deletion manifest file" in error_message:
            reason_codes.append("stale_script_reference_deletion_manifest_missing")
        else:
            reason_codes.append("stale_script_reference_argument_invalid")
        findings = []
        deleted_script_paths = []
        enforced_deleted_script_paths = []
        scan_files = []

    reason_codes = sorted(set(reason_codes))
    status = "ok" if not reason_codes else "fail"
    final_decision = "GO" if status == "ok" else "NO-GO"
    reason_codes_value = "none" if not reason_codes else ",".join(reason_codes)
    report_payload = {
        "schema_version": CHECK_REPORT_SCHEMA_VERSION,
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": reason_codes_value,
        "metrics": {
            "deletion_entry_count": len(deleted_script_paths),
            "enforced_deletion_entry_count": len(enforced_deleted_script_paths),
            "scan_root_count": len(scan_roots),
            "scanned_file_count": len(scan_files),
            "stale_reference_count": len(findings),
        },
        "stale_references": findings,
    }
    try:
        write_json(output_json_path, report_payload)
    except CheckerError:
        print("status=fail")
        print("final_decision=NO-GO")
        print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
        print("reason_codes=stale_script_reference_output_write_failed")
        print(f"reason_codes_csv={REASON_CODES_CSV}")
        print(f"deletion_entry_count={len(deleted_script_paths)}")
        print(f"enforced_deletion_entry_count={len(enforced_deleted_script_paths)}")
        print(f"scan_root_count={len(scan_roots)}")
        print(f"scanned_file_count={len(scan_files)}")
        print(f"stale_reference_count={len(findings)}")
        return 1

    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes={reason_codes_value}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"deletion_entry_count={len(deleted_script_paths)}")
    print(f"enforced_deletion_entry_count={len(enforced_deleted_script_paths)}")
    print(f"scan_root_count={len(scan_roots)}")
    print(f"scanned_file_count={len(scan_files)}")
    print(f"stale_reference_count={len(findings)}")
    print(f"output_json={output_json_path}")

    if status != "ok":
        for finding in findings:
            print(
                "stale_reference="
                f"{finding['reference_file']}::{finding['script_path']}",
                file=sys.stderr,
            )
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
