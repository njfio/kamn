#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REASON_TAXONOMY_VERSION="kamn.ci.specs-index-coverage-reason-taxonomy.v1"
REASON_CODES_CSV="specs_index_duplicate_entry,specs_index_index_missing,specs_index_invalid_argument,specs_index_missing_entry,specs_index_missing_shard,specs_index_missing_shards_marker,specs_index_output_json_required,specs_index_output_write_failed,specs_index_repo_root_missing,specs_index_specs_root_missing,specs_index_unknown_entry"

repo_root="$ROOT_DIR"
output_json=""

emit_result() {
  local status="$1"
  local final_decision="$2"
  local reason_codes="$3"
  local top_level_spec_count="$4"
  local indexed_spec_count="$5"

  echo "status=$status"
  echo "final_decision=$final_decision"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=$reason_codes"
  echo "reason_codes_csv=$REASON_CODES_CSV"
  echo "top_level_spec_count=$top_level_spec_count"
  echo "indexed_spec_count=$indexed_spec_count"
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
      emit_result "fail" "NO-GO" "specs_index_invalid_argument" "unknown" "unknown"
      exit 1
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  emit_result "fail" "NO-GO" "specs_index_output_json_required" "unknown" "unknown"
  exit 1
fi

if [[ ! -d "$repo_root" ]]; then
  emit_result "fail" "NO-GO" "specs_index_repo_root_missing" "0" "0"
  exit 1
fi

if [[ ! -d "$repo_root/specs" ]]; then
  emit_result "fail" "NO-GO" "specs_index_specs_root_missing" "0" "0"
  exit 1
fi

if [[ ! -f "$repo_root/specs/INDEX.md" ]]; then
  emit_result "fail" "NO-GO" "specs_index_index_missing" "0" "0"
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
tmp_report="$tmp_dir/specs-index-coverage-report.json"

python3 - "$repo_root" "$tmp_report" <<'PY'
import json
import re
import sys
from pathlib import Path

repo_root = Path(sys.argv[1]).resolve()
report_path = Path(sys.argv[2]).resolve()
specs_root = repo_root / "specs"
index_doc = specs_root / "INDEX.md"

fail_reasons: list[str] = []
index_body = index_doc.read_text(encoding="utf-8", errors="ignore")
marker_match = re.search(r"^specs_index_shards_csv=([^\n]+)$", index_body, flags=re.MULTILINE)
shard_paths = []
if not marker_match:
    fail_reasons.append("specs_index_missing_shards_marker")
else:
    shard_paths = [item.strip() for item in marker_match.group(1).split(",") if item.strip()]

entries: list[str] = []
for shard_rel in shard_paths:
    shard_path = repo_root / shard_rel
    if not shard_path.is_file():
        fail_reasons.append("specs_index_missing_shard")
        continue
    shard_body = shard_path.read_text(encoding="utf-8", errors="ignore")
    entries.extend(re.findall(r"\]\(\.\./([^)]+\.md)\)", shard_body))

top_level_specs = sorted(
    p.name for p in specs_root.glob("*.md") if p.name != "INDEX.md"
)
entry_set = set(entries)
missing_entries = sorted(set(top_level_specs) - entry_set)
unknown_entries = sorted(entry_set - set(top_level_specs))
duplicate_entries = sorted(name for name in entry_set if entries.count(name) > 1)

if missing_entries:
    fail_reasons.append("specs_index_missing_entry")
if unknown_entries:
    fail_reasons.append("specs_index_unknown_entry")
if duplicate_entries:
    fail_reasons.append("specs_index_duplicate_entry")

reason_codes = "none" if not fail_reasons else ",".join(sorted(set(fail_reasons)))
status = "ok" if reason_codes == "none" else "fail"
final_decision = "GO" if status == "ok" else "NO-GO"

payload = {
    "schema_version": "kamn.ci.specs-index-coverage-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_taxonomy_version": "kamn.ci.specs-index-coverage-reason-taxonomy.v1",
    "reason_codes": reason_codes,
    "metrics": {
        "top_level_spec_count": len(top_level_specs),
        "indexed_spec_count": len(entry_set),
    },
    "details": {
        "missing_entries": missing_entries,
        "unknown_entries": unknown_entries,
        "duplicate_entries": duplicate_entries,
    },
}
report_path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY

read_report_field() {
  local field="$1"
  local fallback="$2"
  python3 - "$tmp_report" "$field" "$fallback" <<'PY'
import json
import sys
from pathlib import Path

payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
value = payload
for key in sys.argv[2].split("."):
    if isinstance(value, dict) and key in value:
        value = value[key]
    else:
        value = sys.argv[3]
        break
print(value if value is not None else sys.argv[3])
PY
}

status="$(read_report_field "status" "fail")"
final_decision="$(read_report_field "final_decision" "NO-GO")"
reason_codes="$(read_report_field "reason_codes" "specs_index_index_missing")"
top_level_spec_count="$(read_report_field "metrics.top_level_spec_count" "unknown")"
indexed_spec_count="$(read_report_field "metrics.indexed_spec_count" "unknown")"

mkdir -p "$(dirname "$output_json")"
if ! cp "$tmp_report" "$output_json"; then
  emit_result "fail" "NO-GO" "specs_index_output_write_failed" "$top_level_spec_count" "$indexed_spec_count"
  exit 1
fi

emit_result "$status" "$final_decision" "$reason_codes" "$top_level_spec_count" "$indexed_spec_count"

if [[ "$status" != "ok" ]]; then
  exit 1
fi
