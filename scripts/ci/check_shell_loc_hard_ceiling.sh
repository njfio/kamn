#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEFAULT_CEILING_FILE="$ROOT_DIR/.ci/shell-loc-hard-ceiling.env"
REASON_TAXONOMY_VERSION="kamn.ci.shell-loc-hard-ceiling-reason-taxonomy.v1"
REASON_CODES_CSV="shell_loc_hard_ceiling_argument_invalid,shell_loc_hard_ceiling_ceiling_file_missing,shell_loc_hard_ceiling_ceiling_key_missing,shell_loc_hard_ceiling_ceiling_value_invalid,shell_loc_hard_ceiling_exceeded,shell_loc_hard_ceiling_git_ls_failed,shell_loc_hard_ceiling_metric_invalid,shell_loc_hard_ceiling_output_json_required,shell_loc_hard_ceiling_output_write_failed"

repo_root="$ROOT_DIR"
ceiling_file="$DEFAULT_CEILING_FILE"
output_json=""

emit_result() {
  local status="$1"
  local final_decision="$2"
  local reason_codes="$3"
  local shell_line_total="$4"
  local hard_shell_loc_max="$5"
  local tracked_shell_file_count="$6"
  local delta_to_hard_shell_loc_max="$7"

  echo "status=$status"
  echo "final_decision=$final_decision"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=$reason_codes"
  echo "reason_codes_csv=$REASON_CODES_CSV"
  echo "shell_line_total=$shell_line_total"
  echo "hard_shell_loc_max=$hard_shell_loc_max"
  echo "tracked_shell_file_count=$tracked_shell_file_count"
  echo "delta_to_hard_shell_loc_max=$delta_to_hard_shell_loc_max"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      repo_root="${2:-}"
      shift 2
      ;;
    --ceiling-file)
      ceiling_file="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      emit_result "fail" "NO-GO" "shell_loc_hard_ceiling_argument_invalid" "unknown" "unknown" "unknown" "unknown"
      exit 1
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  emit_result "fail" "NO-GO" "shell_loc_hard_ceiling_output_json_required" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -d "$repo_root" ]]; then
  emit_result "fail" "NO-GO" "shell_loc_hard_ceiling_argument_invalid" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -f "$ceiling_file" ]]; then
  emit_result "fail" "NO-GO" "shell_loc_hard_ceiling_ceiling_file_missing" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

hard_shell_loc_max_raw="$(grep -E '^HARD_SHELL_LOC_MAX=' "$ceiling_file" | head -n 1 | cut -d= -f2- || true)"
if [[ -z "$hard_shell_loc_max_raw" ]]; then
  emit_result "fail" "NO-GO" "shell_loc_hard_ceiling_ceiling_key_missing" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi
if [[ ! "$hard_shell_loc_max_raw" =~ ^[0-9]+$ ]]; then
  emit_result "fail" "NO-GO" "shell_loc_hard_ceiling_ceiling_value_invalid" "unknown" "$hard_shell_loc_max_raw" "unknown" "unknown"
  exit 1
fi

tmp_dir="$(mktemp -d)"
trap 'rm -rf "$tmp_dir"' EXIT
tmp_metrics_json="$tmp_dir/shell-loc-metrics.json"

set +e
python3 - "$repo_root" "$hard_shell_loc_max_raw" "$tmp_metrics_json" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

repo_root = Path(sys.argv[1]).resolve()
hard_shell_loc_max = int(sys.argv[2])
tmp_metrics_json = Path(sys.argv[3])

try:
    tracked = subprocess.check_output(
        ["git", "-C", str(repo_root), "ls-files", "*.sh"],
        text=True,
    ).splitlines()
except subprocess.CalledProcessError:
    payload = {"status": "fail", "reason": "shell_loc_hard_ceiling_git_ls_failed"}
    tmp_metrics_json.write_text(json.dumps(payload), encoding="utf-8")
    raise SystemExit(1)

shell_files: list[Path] = []
for rel in tracked:
    path = (repo_root / rel).resolve()
    original = repo_root / rel
    if original.is_symlink():
        continue
    if not original.is_file():
        continue
    shell_files.append(original)

shell_line_total = 0
for path in shell_files:
    shell_line_total += sum(1 for _ in path.open("r", encoding="utf-8", errors="ignore"))

delta = hard_shell_loc_max - shell_line_total
status = "ok"
final_decision = "GO"
reason_codes = "none"
if shell_line_total > hard_shell_loc_max:
    status = "fail"
    final_decision = "NO-GO"
    reason_codes = "shell_loc_hard_ceiling_exceeded"

payload = {
    "schema_version": "kamn.ci.shell-loc-hard-ceiling-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_taxonomy_version": "kamn.ci.shell-loc-hard-ceiling-reason-taxonomy.v1",
    "reason_codes": reason_codes,
    "metrics": {
        "shell_line_total": shell_line_total,
        "hard_shell_loc_max": hard_shell_loc_max,
        "tracked_shell_file_count": len(shell_files),
        "delta_to_hard_shell_loc_max": delta,
    },
}
tmp_metrics_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
PY
py_exit=$?
set -e

if [[ ! -f "$tmp_metrics_json" ]]; then
  emit_result "fail" "NO-GO" "shell_loc_hard_ceiling_metric_invalid" "unknown" "$hard_shell_loc_max_raw" "unknown" "unknown"
  exit 1
fi

if [[ "$py_exit" -ne 0 ]]; then
  reason_from_python="$(python3 - "$tmp_metrics_json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("reason", "shell_loc_hard_ceiling_metric_invalid"))
PY
)"
  emit_result "fail" "NO-GO" "$reason_from_python" "unknown" "$hard_shell_loc_max_raw" "unknown" "unknown"
  exit 1
fi

status="$(python3 - "$tmp_metrics_json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("status", "fail"))
PY
)"
final_decision="$(python3 - "$tmp_metrics_json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("final_decision", "NO-GO"))
PY
)"
reason_codes="$(python3 - "$tmp_metrics_json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("reason_codes", "shell_loc_hard_ceiling_metric_invalid"))
PY
)"
shell_line_total="$(python3 - "$tmp_metrics_json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("metrics", {}).get("shell_line_total", "unknown"))
PY
)"
hard_shell_loc_max="$(python3 - "$tmp_metrics_json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("metrics", {}).get("hard_shell_loc_max", "unknown"))
PY
)"
tracked_shell_file_count="$(python3 - "$tmp_metrics_json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("metrics", {}).get("tracked_shell_file_count", "unknown"))
PY
)"
delta_to_hard_shell_loc_max="$(python3 - "$tmp_metrics_json" <<'PY'
import json
import sys
from pathlib import Path
payload = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
print(payload.get("metrics", {}).get("delta_to_hard_shell_loc_max", "unknown"))
PY
)"

mkdir -p "$(dirname "$output_json")"
if ! cp "$tmp_metrics_json" "$output_json"; then
  emit_result "fail" "NO-GO" "shell_loc_hard_ceiling_output_write_failed" "$shell_line_total" "$hard_shell_loc_max" "$tracked_shell_file_count" "$delta_to_hard_shell_loc_max"
  exit 1
fi

emit_result "$status" "$final_decision" "$reason_codes" "$shell_line_total" "$hard_shell_loc_max" "$tracked_shell_file_count" "$delta_to_hard_shell_loc_max"

if [[ "$status" != "ok" ]]; then
  exit 1
fi

