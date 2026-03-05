#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
REASON_TAXONOMY_VERSION="kamn.ci.spec-phase6-evidence-policy-reason-taxonomy.v1"
REASON_CODES_CSV="spec_phase6_invalid_argument,spec_phase6_missing_execution_markers,spec_phase6_missing_section,spec_phase6_output_json_required,spec_phase6_output_write_failed,spec_phase6_repo_root_missing,spec_phase6_specs_root_missing"

repo_root="$ROOT_DIR"
output_json=""

emit_result() {
  local status="$1"
  local final_decision="$2"
  local reason_codes="$3"
  local scanned_spec_count="$4"
  local closure_ready_spec_count="$5"
  local phase6_compliant_spec_count="$6"

  echo "status=$status"
  echo "final_decision=$final_decision"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=$reason_codes"
  echo "reason_codes_csv=$REASON_CODES_CSV"
  echo "scanned_spec_count=$scanned_spec_count"
  echo "closure_ready_spec_count=$closure_ready_spec_count"
  echo "phase6_compliant_spec_count=$phase6_compliant_spec_count"
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
      emit_result "fail" "NO-GO" "spec_phase6_invalid_argument" "unknown" "unknown" "unknown"
      exit 1
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  emit_result "fail" "NO-GO" "spec_phase6_output_json_required" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -d "$repo_root" ]]; then
  emit_result "fail" "NO-GO" "spec_phase6_repo_root_missing" "0" "0" "0"
  exit 1
fi

if [[ ! -d "$repo_root/specs" ]]; then
  emit_result "fail" "NO-GO" "spec_phase6_specs_root_missing" "0" "0" "0"
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
tmp_report="$tmp_dir/spec-phase6-evidence-policy-report.json"

python3 - "$repo_root" "$tmp_report" <<'PY'
import json
import re
import sys
from pathlib import Path

repo_root = Path(sys.argv[1]).resolve()
report_path = Path(sys.argv[2]).resolve()
specs_root = repo_root / "specs"

fail_reasons: list[str] = []
scanned_spec_count = 0
closure_ready_spec_count = 0
phase6_compliant_spec_count = 0

if not specs_root.is_dir():
    fail_reasons.append("spec_phase6_specs_root_missing")
else:
    for spec_path in sorted(specs_root.glob("*.md")):
        scanned_spec_count += 1
        body = spec_path.read_text(encoding="utf-8", errors="ignore")
        status_match = re.search(
            r"^(?:-\s*)?Status:\s*`?([^`\n]+)`?\s*$",
            body,
            flags=re.MULTILINE | re.IGNORECASE,
        )
        status_value = status_match.group(1).strip().lower() if status_match else ""
        if status_value != "implemented":
            continue

        closure_ready_spec_count += 1
        section_match = re.search(
            r"^##\s*Phase\s*6\s*integration\s*evidence\b",
            body,
            flags=re.MULTILINE | re.IGNORECASE,
        )
        if not section_match:
            fail_reasons.append("spec_phase6_missing_section")
            continue

        section_start = section_match.end()
        next_heading_match = re.search(r"^##\s+", body[section_start:], flags=re.MULTILINE)
        if next_heading_match:
            section_end = section_start + next_heading_match.start()
        else:
            section_end = len(body)
        section_body = body[section_start:section_end]

        has_executed_marker = bool(
            re.search(r"^\s*-\s*Executed:\s*$", section_body, flags=re.MULTILINE | re.IGNORECASE)
        )
        has_command_marker = bool(
            re.search(r"^\s*-\s+`[^`]+`\s*$", section_body, flags=re.MULTILINE)
        )

        if not (has_executed_marker and has_command_marker):
            fail_reasons.append("spec_phase6_missing_execution_markers")
            continue

        phase6_compliant_spec_count += 1

dedup_reasons = sorted(set(fail_reasons))
status = "ok" if not dedup_reasons else "fail"
final_decision = "GO" if status == "ok" else "NO-GO"
reason_codes = "none" if not dedup_reasons else ",".join(dedup_reasons)

payload = {
    "schema_version": "kamn.ci.spec-phase6-evidence-policy-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_taxonomy_version": "kamn.ci.spec-phase6-evidence-policy-reason-taxonomy.v1",
    "reason_codes": reason_codes,
    "metrics": {
        "scanned_spec_count": scanned_spec_count,
        "closure_ready_spec_count": closure_ready_spec_count,
        "phase6_compliant_spec_count": phase6_compliant_spec_count,
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
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("reason_codes", "spec_phase6_repo_root_missing"))
PY
)"
scanned_spec_count="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("metrics", {}).get("scanned_spec_count", "unknown"))
PY
)"
closure_ready_spec_count="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("metrics", {}).get("closure_ready_spec_count", "unknown"))
PY
)"
phase6_compliant_spec_count="$(python3 - "$tmp_report" <<'PY'
import json
import sys
from pathlib import Path
print(json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")).get("metrics", {}).get("phase6_compliant_spec_count", "unknown"))
PY
)"

mkdir -p "$(dirname "$output_json")"
if ! cp "$tmp_report" "$output_json"; then
  emit_result "fail" "NO-GO" "spec_phase6_output_write_failed" "$scanned_spec_count" "$closure_ready_spec_count" "$phase6_compliant_spec_count"
  exit 1
fi

emit_result "$status" "$final_decision" "$reason_codes" "$scanned_spec_count" "$closure_ready_spec_count" "$phase6_compliant_spec_count"

if [[ "$status" != "ok" ]]; then
  exit 1
fi
