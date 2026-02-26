#!/usr/bin/env python3
"""Fail-closed review-document freeze checker for PR fast gate."""

from __future__ import annotations

import argparse
import json
from pathlib import Path
from typing import Dict, List, Sequence

SCHEMA_VERSION = "kamn.ci.review-document-freeze-gate-report.v1"
REASON_TAXONOMY_VERSION = "kamn.ci.review-document-freeze-gate-reason-taxonomy.v1"
FREEZE_SCHEMA_VERSION = "kamn.review.review-document-freeze-manifest.v1"

REASON_CHANGED_FILES_MISSING = "review_document_freeze_changed_files_missing"
REASON_MANIFEST_MISSING = "review_document_freeze_manifest_missing"
REASON_MANIFEST_INVALID = "review_document_freeze_manifest_invalid"
REASON_VIOLATION_DETECTED = "review_document_freeze_violation_detected"

FREEZE_ENTRIES_KEY = "review_document_freeze_entries_csv"
FREEZE_SCHEMA_KEY = "review_document_freeze_manifest_schema_version"


class CheckerFailure(Exception):
    """Deterministic checker failure with explicit reason code."""

    def __init__(self, reason_code: str, detail: str) -> None:
        super().__init__(detail)
        self.reason_code = reason_code
        self.detail = detail


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Block modifications to frozen review documents declared in freeze manifest."
    )
    parser.add_argument(
        "--changed-files-file",
        required=True,
        help="Path to newline-delimited changed file list (repo-relative).",
    )
    parser.add_argument(
        "--freeze-manifest",
        default="docs/review/review-document-freeze.manifest",
        help="Freeze manifest path containing review_document_freeze_entries_csv.",
    )
    parser.add_argument(
        "--output-json",
        default="ci-review-document-freeze.json",
        help="Output path for schema-versioned checker report.",
    )
    return parser.parse_args()


def normalize_repo_path(path: str) -> str:
    normalized = path.strip().replace("\\", "/")
    while normalized.startswith("./"):
        normalized = normalized[2:]
    return normalized


def read_changed_files(path: Path) -> List[str]:
    if not path.is_file():
        raise CheckerFailure(
            REASON_CHANGED_FILES_MISSING,
            f"changed files file missing: {path}",
        )
    contents = path.read_text(encoding="utf-8")
    return [normalize_repo_path(line) for line in contents.splitlines() if line.strip()]


def parse_key_value_manifest(path: Path) -> Dict[str, str]:
    if not path.is_file():
        raise CheckerFailure(REASON_MANIFEST_MISSING, f"freeze manifest missing: {path}")
    payload = path.read_text(encoding="utf-8")
    manifest: Dict[str, str] = {}
    for raw_line in payload.splitlines():
        line = raw_line.strip()
        if not line or line.startswith("#"):
            continue
        if "=" not in line:
            continue
        key, value = line.split("=", 1)
        manifest[key.strip()] = value.strip()
    return manifest


def parse_frozen_entries(manifest: Dict[str, str]) -> Sequence[str]:
    schema = manifest.get(FREEZE_SCHEMA_KEY, "")
    if schema != FREEZE_SCHEMA_VERSION:
        raise CheckerFailure(
            REASON_MANIFEST_INVALID,
            f"freeze manifest schema mismatch: expected {FREEZE_SCHEMA_VERSION}, got {schema or '<missing>'}",
        )
    entries_csv = manifest.get(FREEZE_ENTRIES_KEY, "")
    if not entries_csv:
        raise CheckerFailure(
            REASON_MANIFEST_INVALID,
            f"freeze manifest missing non-empty {FREEZE_ENTRIES_KEY}",
        )
    entries = [entry.strip() for entry in entries_csv.split(",") if entry.strip()]
    if not entries:
        raise CheckerFailure(
            REASON_MANIFEST_INVALID,
            "freeze manifest has no frozen review entries",
        )
    return entries


def build_report(
    changed_files: Sequence[str],
    frozen_entries: Sequence[str],
    freeze_manifest: Path,
) -> Dict[str, object]:
    frozen_paths = {
        normalize_repo_path(f"docs/review/{entry}") for entry in frozen_entries
    }
    blocked = sorted(path for path in changed_files if normalize_repo_path(path) in frozen_paths)
    reason_codes: List[str] = []
    if blocked:
        reason_codes.append(REASON_VIOLATION_DETECTED)

    status = "ok" if not reason_codes else "violation"
    reason_codes_csv = "none" if not reason_codes else ",".join(reason_codes)
    return {
        "schema_version": SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": reason_codes_csv,
        "status": status,
        "freeze_manifest_path": normalize_repo_path(str(freeze_manifest)),
        "changed_file_count": len(changed_files),
        "frozen_entry_count": len(frozen_entries),
        "blocked_changed_files": blocked,
    }


def write_report(path: Path, report: Dict[str, object]) -> None:
    path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")


def emit_stdout(report: Dict[str, object]) -> None:
    print(f"status={report['status']}")
    print(f"reason_taxonomy_version={report['reason_taxonomy_version']}")
    print(f"reason_codes_csv={report['reason_codes_csv']}")
    print(f"freeze_manifest_path={report['freeze_manifest_path']}")
    print(f"changed_file_count={report['changed_file_count']}")
    print(f"frozen_entry_count={report['frozen_entry_count']}")
    blocked = report["blocked_changed_files"]
    if isinstance(blocked, list) and blocked:
        print(f"blocked_changed_files_csv={','.join(str(value) for value in blocked)}")
    else:
        print("blocked_changed_files_csv=none")


def failure_report(reason_code: str, detail: str, freeze_manifest: Path) -> Dict[str, object]:
    return {
        "schema_version": SCHEMA_VERSION,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes_csv": reason_code,
        "status": "violation",
        "freeze_manifest_path": normalize_repo_path(str(freeze_manifest)),
        "error": detail,
        "changed_file_count": 0,
        "frozen_entry_count": 0,
        "blocked_changed_files": [],
    }


def main() -> int:
    args = parse_args()
    output_path = Path(args.output_json)
    freeze_manifest_path = Path(args.freeze_manifest)
    try:
        changed_files = read_changed_files(Path(args.changed_files_file))
        manifest = parse_key_value_manifest(freeze_manifest_path)
        frozen_entries = parse_frozen_entries(manifest)
        report = build_report(changed_files, frozen_entries, freeze_manifest_path)
        write_report(output_path, report)
        emit_stdout(report)
        return 0 if report["status"] == "ok" else 1
    except CheckerFailure as error:
        report = failure_report(error.reason_code, error.detail, freeze_manifest_path)
        write_report(output_path, report)
        emit_stdout(report)
        return 1


if __name__ == "__main__":
    raise SystemExit(main())
