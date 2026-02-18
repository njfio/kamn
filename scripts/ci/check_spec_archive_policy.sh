#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REASON_TAXONOMY_VERSION="kamn.ci.spec-archive-policy-reason-taxonomy.v1"
REASON_CODES_CSV="spec_archive_entry_missing_file,spec_archive_index_count_mismatch,spec_archive_index_entry_missing,spec_archive_index_missing,spec_archive_output_json_required,spec_archive_output_write_failed,spec_archive_pointer_invalid,spec_archive_pointer_missing,spec_archive_root_missing,spec_archive_status_not_implemented"

repo_root="$ROOT_DIR"
output_json=""

emit_result() {
  local status="$1"
  local final_decision="$2"
  local reason_codes="$3"
  local archived_issue_count="$4"
  local pointer_count="$5"
  local index_entry_count="$6"

  echo "status=$status"
  echo "final_decision=$final_decision"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=$reason_codes"
  echo "reason_codes_csv=$REASON_CODES_CSV"
  echo "archived_issue_count=$archived_issue_count"
  echo "pointer_count=$pointer_count"
  echo "index_entry_count=$index_entry_count"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      repo_root="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      emit_result "fail" "NO-GO" "spec_archive_root_missing" "unknown" "unknown" "unknown"
      exit 1
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  emit_result "fail" "NO-GO" "spec_archive_output_json_required" "unknown" "unknown" "unknown"
  exit 1
fi

archive_root="$repo_root/specs/archive"
if [[ ! -d "$archive_root" ]]; then
  emit_result "fail" "NO-GO" "spec_archive_root_missing" "0" "0" "0"
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
tmp_report="$tmp_dir/spec-archive-policy-report.json"

python3 - "$repo_root" "$tmp_report" <<'PY'
import json
import re
import sys
from pathlib import Path

repo_root = Path(sys.argv[1]).resolve()
report_path = Path(sys.argv[2]).resolve()
archive_root = repo_root / "specs" / "archive"

fail_reasons: list[str] = []
archived_issue_count = 0
pointer_count = 0
index_entry_count = 0

if not archive_root.is_dir():
    fail_reasons.append("spec_archive_root_missing")
else:
    index_file = archive_root / "index.md"
    index_body = ""
    declared_index_count = None
    if not index_file.is_file():
        fail_reasons.append("spec_archive_index_missing")
    else:
        index_body = index_file.read_text(encoding="utf-8", errors="ignore")
        declared_match = re.search(r"archived_issue_count:\s*(\d+)", index_body)
        if declared_match:
            declared_index_count = int(declared_match.group(1))
        index_entry_count = len(re.findall(r"^\|\s*\d+\s*\|", index_body, flags=re.MULTILINE))

    for issue_dir in sorted(archive_root.iterdir()):
        if not issue_dir.is_dir():
            continue
        if not re.fullmatch(r"\d+", issue_dir.name):
            continue
        archived_issue_count += 1
        required = ("spec.md", "plan.md", "tasks.md")
        for required_file in required:
            if not (issue_dir / required_file).is_file():
                fail_reasons.append("spec_archive_entry_missing_file")
        spec_file = issue_dir / "spec.md"
        if spec_file.is_file():
            body = spec_file.read_text(encoding="utf-8", errors="ignore")
            status_match = re.search(r"^- Status:\s*`?([^`\n]+)`?\s*$", body, flags=re.MULTILINE)
            status_value = status_match.group(1).strip().lower() if status_match else ""
            if status_value != "implemented":
                fail_reasons.append("spec_archive_status_not_implemented")
        pointer = repo_root / "specs" / issue_dir.name / "ARCHIVED.md"
        if not pointer.is_file():
            fail_reasons.append("spec_archive_pointer_missing")
        else:
            pointer_count += 1
            expected_line = f"archive_path: specs/archive/{issue_dir.name}"
            pointer_body = pointer.read_text(encoding="utf-8", errors="ignore")
            if expected_line not in pointer_body:
                fail_reasons.append("spec_archive_pointer_invalid")
        if index_body:
            archive_path = f"specs/archive/{issue_dir.name}"
            pointer_path = f"specs/{issue_dir.name}/ARCHIVED.md"
            if (
                f"| {issue_dir.name} |" not in index_body
                or archive_path not in index_body
                or pointer_path not in index_body
            ):
                fail_reasons.append("spec_archive_index_entry_missing")

    if declared_index_count is not None and declared_index_count != archived_issue_count:
        fail_reasons.append("spec_archive_index_count_mismatch")

dedup_reasons = sorted(set(fail_reasons))
status = "ok" if not dedup_reasons else "fail"
final_decision = "GO" if status == "ok" else "NO-GO"
reason_codes = "none" if not dedup_reasons else ",".join(dedup_reasons)

payload = {
    "schema_version": "kamn.ci.spec-archive-policy-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_taxonomy_version": "kamn.ci.spec-archive-policy-reason-taxonomy.v1",
    "reason_codes": reason_codes,
    "metrics": {
        "archived_issue_count": archived_issue_count,
        "pointer_count": pointer_count,
        "index_entry_count": index_entry_count,
    },
}
report_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

status="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("status", "fail"))
PY
)"
final_decision="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("final_decision", "NO-GO"))
PY
)"
reason_codes="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("reason_codes", "spec_archive_root_missing"))
PY
)"
archived_issue_count="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("metrics", {}).get("archived_issue_count", "unknown"))
PY
)"
pointer_count="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("metrics", {}).get("pointer_count", "unknown"))
PY
)"
index_entry_count="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("metrics", {}).get("index_entry_count", "unknown"))
PY
)"

mkdir -p "$(dirname "$output_json")"
if ! cp "$tmp_report" "$output_json"; then
  emit_result "fail" "NO-GO" "spec_archive_output_write_failed" "$archived_issue_count" "$pointer_count" "$index_entry_count"
  exit 1
fi

emit_result "$status" "$final_decision" "$reason_codes" "$archived_issue_count" "$pointer_count" "$index_entry_count"

if [[ "$status" != "ok" ]]; then
  exit 1
fi
