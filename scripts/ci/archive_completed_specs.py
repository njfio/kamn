#!/usr/bin/env python3
"""Archive implemented issue specs and emit deterministic migration report markers."""

from __future__ import annotations

import argparse
import json
import re
import shutil
import sys
from dataclasses import dataclass
from datetime import date
from pathlib import Path
from typing import Dict, List


REASON_TAXONOMY_VERSION = "kamn.ci.spec-archive-tool-reason-taxonomy.v1"
REASON_CODES_CSV = ",".join(
    [
        "spec_archive_tool_archive_target_exists",
        "spec_archive_tool_argument_invalid",
        "spec_archive_tool_issue_id_invalid",
        "spec_archive_tool_issue_missing",
        "spec_archive_tool_output_json_required",
        "spec_archive_tool_output_write_failed",
        "spec_archive_tool_required_file_missing",
        "spec_archive_tool_status_not_implemented",
    ]
)
POINTER_RATIONALE = "Completed issue spec archived per shell-loc governance policy."


@dataclass
class IssuePlan:
    issue_id: str
    source_dir: Path
    target_dir: Path
    title: str


def add_reason(reasons: List[str], code: str) -> None:
    if code not in reasons:
        reasons.append(code)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Archive implemented issue specs into specs/archive and maintain index report."
    )
    parser.add_argument("--repo-root", default=".", help="Repository root (default: current directory)")
    parser.add_argument(
        "--issue-id",
        dest="issue_ids",
        action="append",
        help="Issue id to archive (repeatable)",
    )
    parser.add_argument("--apply", action="store_true", help="Apply archive move (default: dry-run)")
    parser.add_argument(
        "--archived-on",
        default=date.today().isoformat(),
        help="Archived-on date marker (YYYY-MM-DD, default: today)",
    )
    parser.add_argument("--output-json", required=True, help="Path to write JSON report")
    return parser.parse_args()


def load_existing_index_entries(index_path: Path) -> Dict[str, Dict[str, str]]:
    entries: Dict[str, Dict[str, str]] = {}
    if not index_path.is_file():
        return entries

    row_re = re.compile(
        r"^\|\s*(\d+)\s*\|\s*(.*?)\s*\|\s*([0-9]{4}-[0-9]{2}-[0-9]{2})\s*\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|$"
    )
    for line in index_path.read_text(encoding="utf-8", errors="ignore").splitlines():
        match = row_re.match(line.strip())
        if not match:
            continue
        issue_id, title, archived_on, archive_path, pointer_path = match.groups()
        entries[issue_id] = {
            "title": title,
            "archived_on": archived_on,
            "archive_path": archive_path,
            "pointer_path": pointer_path,
        }
    return entries


def render_index(entries: Dict[str, Dict[str, str]]) -> str:
    ordered_ids = sorted(entries.keys(), key=lambda raw: int(raw))
    lines = [
        "# Archived Spec Index",
        "",
        "- schema_version: kamn.specs.archive-index-report.v1",
        "- migration_wave_id: r27-44-initial-archive-wave",
        f"- archived_on: {date.today().isoformat()}",
        f"- archived_issue_count: {len(ordered_ids)}",
        "",
        "| issue_id | title | archived_on | archive_path | pointer_path |",
        "|---|---|---|---|---|",
    ]
    for issue_id in ordered_ids:
        entry = entries[issue_id]
        lines.append(
            f"| {issue_id} | {entry['title']} | {entry['archived_on']} | "
            f"`{entry['archive_path']}` | `{entry['pointer_path']}` |"
        )
    return "\n".join(lines) + "\n"


def emit_markers(
    *,
    status: str,
    final_decision: str,
    reason_codes: List[str],
    mode: str,
    requested_issue_count: int,
    archived_issue_count: int,
    index_entry_count: int,
    output_json: Path,
) -> int:
    reason_codes_value = "none" if not reason_codes else ",".join(reason_codes)
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes={reason_codes_value}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"mode={mode}")
    print(f"requested_issue_count={requested_issue_count}")
    print(f"archived_issue_count={archived_issue_count}")
    print(f"index_entry_count={index_entry_count}")
    print(f"output_json={output_json}")
    return 0 if status == "ok" else 1


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    output_json = Path(args.output_json).resolve()
    mode = "apply" if args.apply else "dry-run"
    requested_issue_ids = args.issue_ids or []
    reasons: List[str] = []

    if not requested_issue_ids:
        add_reason(reasons, "spec_archive_tool_argument_invalid")
    if not re.fullmatch(r"\d{4}-\d{2}-\d{2}", args.archived_on):
        add_reason(reasons, "spec_archive_tool_argument_invalid")

    plans: List[IssuePlan] = []
    for raw_issue_id in requested_issue_ids:
        if not re.fullmatch(r"\d+", raw_issue_id):
            add_reason(reasons, "spec_archive_tool_issue_id_invalid")
            continue
        issue_id = str(int(raw_issue_id))
        source_dir = repo_root / "specs" / issue_id
        if not source_dir.is_dir():
            add_reason(reasons, "spec_archive_tool_issue_missing")
            continue

        required_files = [source_dir / "spec.md", source_dir / "plan.md", source_dir / "tasks.md"]
        missing_file = next((path for path in required_files if not path.is_file()), None)
        if missing_file is not None:
            add_reason(reasons, "spec_archive_tool_required_file_missing")
            continue

        spec_body = (source_dir / "spec.md").read_text(encoding="utf-8", errors="ignore")
        if "- Status: Implemented" not in spec_body:
            add_reason(reasons, "spec_archive_tool_status_not_implemented")
            continue

        title_match = re.search(r"^- Title:\s*(.+)$", spec_body, flags=re.MULTILINE)
        title = title_match.group(1).strip() if title_match else f"Issue {issue_id}"

        target_dir = repo_root / "specs" / "archive" / issue_id
        if target_dir.exists():
            add_reason(reasons, "spec_archive_tool_archive_target_exists")
            continue

        plans.append(
            IssuePlan(
                issue_id=issue_id,
                source_dir=source_dir,
                target_dir=target_dir,
                title=title,
            )
        )

    existing_entries = load_existing_index_entries(repo_root / "specs" / "archive" / "index.md")
    planned_entries = dict(existing_entries)
    for plan in plans:
        planned_entries[plan.issue_id] = {
            "title": plan.title,
            "archived_on": args.archived_on,
            "archive_path": f"specs/archive/{plan.issue_id}",
            "pointer_path": f"specs/{plan.issue_id}/ARCHIVED.md",
        }

    status = "ok" if not reasons else "fail"
    final_decision = "GO" if status == "ok" else "NO-GO"
    archived_issue_count = 0

    if status == "ok" and args.apply:
        archive_root = repo_root / "specs" / "archive"
        archive_root.mkdir(parents=True, exist_ok=True)
        for plan in plans:
            plan.target_dir.mkdir(parents=True, exist_ok=False)
            for filename in ("spec.md", "plan.md", "tasks.md"):
                shutil.move(str(plan.source_dir / filename), str(plan.target_dir / filename))
            pointer_path = plan.source_dir / "ARCHIVED.md"
            pointer_path.write_text(
                "\n".join(
                    [
                        "# Archived Spec Pointer",
                        "",
                        f"- issue_id: {plan.issue_id}",
                        f"- archived_on: {args.archived_on}",
                        f"- archive_path: specs/archive/{plan.issue_id}",
                        f"- rationale: {POINTER_RATIONALE}",
                        "",
                    ]
                ),
                encoding="utf-8",
            )
            archived_issue_count += 1

        index_path = archive_root / "index.md"
        index_path.write_text(render_index(planned_entries), encoding="utf-8")

    reason_codes = sorted(reasons)
    payload = {
        "schema_version": "kamn.ci.spec-archive-tool-report.v1",
        "status": status,
        "final_decision": final_decision,
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": "none" if not reason_codes else ",".join(reason_codes),
        "metrics": {
            "mode": mode,
            "requested_issue_count": len(requested_issue_ids),
            "archived_issue_count": archived_issue_count,
            "index_entry_count": len(planned_entries),
        },
    }

    try:
        output_json.parent.mkdir(parents=True, exist_ok=True)
        output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    except Exception:
        fail_codes = ["spec_archive_tool_output_write_failed"]
        return emit_markers(
            status="fail",
            final_decision="NO-GO",
            reason_codes=fail_codes,
            mode=mode,
            requested_issue_count=len(requested_issue_ids),
            archived_issue_count=archived_issue_count,
            index_entry_count=len(planned_entries),
            output_json=output_json,
        )

    return emit_markers(
        status=status,
        final_decision=final_decision,
        reason_codes=reason_codes,
        mode=mode,
        requested_issue_count=len(requested_issue_ids),
        archived_issue_count=archived_issue_count,
        index_entry_count=len(planned_entries),
        output_json=output_json,
    )


if __name__ == "__main__":
    sys.exit(main())
