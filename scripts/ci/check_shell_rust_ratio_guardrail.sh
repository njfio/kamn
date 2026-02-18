#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEFAULT_THRESHOLD_FILE="$ROOT_DIR/.ci/shell-rust-ratio-guardrail.env"
REASON_TAXONOMY_VERSION="kamn.ci.shell-rust-ratio-guardrail-reason-taxonomy.v1"
REASON_CODES_CSV="shell_rust_ratio_argument_invalid,shell_rust_ratio_fail_threshold_exceeded,shell_rust_ratio_git_ls_failed,shell_rust_ratio_metric_invalid,shell_rust_ratio_output_json_required,shell_rust_ratio_output_write_failed,shell_rust_ratio_rust_line_total_invalid,shell_rust_ratio_threshold_file_missing,shell_rust_ratio_threshold_key_missing,shell_rust_ratio_threshold_order_invalid,shell_rust_ratio_threshold_value_invalid,shell_rust_ratio_warn_threshold_exceeded"

repo_root="$ROOT_DIR"
threshold_file="$DEFAULT_THRESHOLD_FILE"
output_json=""

emit_result() {
  local status="$1"
  local final_decision="$2"
  local reason_codes="$3"
  local shell_line_total="$4"
  local rust_line_total="$5"
  local shell_to_rust_ratio="$6"
  local warn_shell_rust_ratio_max="$7"
  local fail_shell_rust_ratio_max="$8"
  local tracked_shell_file_count="$9"
  local tracked_rust_file_count="${10}"
  local delta_to_warn_shell_rust_ratio_max="${11}"
  local delta_to_fail_shell_rust_ratio_max="${12}"

  echo "status=$status"
  echo "final_decision=$final_decision"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=$reason_codes"
  echo "reason_codes_csv=$REASON_CODES_CSV"
  echo "shell_line_total=$shell_line_total"
  echo "rust_line_total=$rust_line_total"
  echo "shell_to_rust_ratio=$shell_to_rust_ratio"
  echo "warn_shell_rust_ratio_max=$warn_shell_rust_ratio_max"
  echo "fail_shell_rust_ratio_max=$fail_shell_rust_ratio_max"
  echo "tracked_shell_file_count=$tracked_shell_file_count"
  echo "tracked_rust_file_count=$tracked_rust_file_count"
  echo "delta_to_warn_shell_rust_ratio_max=$delta_to_warn_shell_rust_ratio_max"
  echo "delta_to_fail_shell_rust_ratio_max=$delta_to_fail_shell_rust_ratio_max"
}

while [[ $# -gt 0 ]]; do
  case "$1" in
    --repo-root)
      repo_root="${2:-}"
      shift 2
      ;;
    --threshold-file)
      threshold_file="${2:-}"
      shift 2
      ;;
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    *)
      emit_result "fail" "NO-GO" "shell_rust_ratio_argument_invalid" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown"
      exit 1
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  emit_result "fail" "NO-GO" "shell_rust_ratio_output_json_required" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -d "$repo_root" ]]; then
  emit_result "fail" "NO-GO" "shell_rust_ratio_argument_invalid" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -f "$threshold_file" ]]; then
  emit_result "fail" "NO-GO" "shell_rust_ratio_threshold_file_missing" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown" "unknown"
  exit 1
fi

set +e
python_output="$(python3 - "$repo_root" "$threshold_file" "$output_json" <<'PY'
import json
import subprocess
import sys
from pathlib import Path

REASON_TAXONOMY_VERSION = "kamn.ci.shell-rust-ratio-guardrail-reason-taxonomy.v1"
REASON_CODES_CSV = (
    "shell_rust_ratio_argument_invalid,"
    "shell_rust_ratio_fail_threshold_exceeded,"
    "shell_rust_ratio_git_ls_failed,"
    "shell_rust_ratio_metric_invalid,"
    "shell_rust_ratio_output_json_required,"
    "shell_rust_ratio_output_write_failed,"
    "shell_rust_ratio_rust_line_total_invalid,"
    "shell_rust_ratio_threshold_file_missing,"
    "shell_rust_ratio_threshold_key_missing,"
    "shell_rust_ratio_threshold_order_invalid,"
    "shell_rust_ratio_threshold_value_invalid,"
    "shell_rust_ratio_warn_threshold_exceeded"
)

repo_root = Path(sys.argv[1]).resolve()
threshold_file = Path(sys.argv[2]).resolve()
output_json = Path(sys.argv[3]).resolve()


def emit(
    status: str,
    final_decision: str,
    reason_codes: str,
    *,
    shell_line_total: str,
    rust_line_total: str,
    shell_to_rust_ratio: str,
    warn_shell_rust_ratio_max: str,
    fail_shell_rust_ratio_max: str,
    tracked_shell_file_count: str,
    tracked_rust_file_count: str,
    delta_to_warn_shell_rust_ratio_max: str,
    delta_to_fail_shell_rust_ratio_max: str,
):
    print(f"status={status}")
    print(f"final_decision={final_decision}")
    print(f"reason_taxonomy_version={REASON_TAXONOMY_VERSION}")
    print(f"reason_codes={reason_codes}")
    print(f"reason_codes_csv={REASON_CODES_CSV}")
    print(f"shell_line_total={shell_line_total}")
    print(f"rust_line_total={rust_line_total}")
    print(f"shell_to_rust_ratio={shell_to_rust_ratio}")
    print(f"warn_shell_rust_ratio_max={warn_shell_rust_ratio_max}")
    print(f"fail_shell_rust_ratio_max={fail_shell_rust_ratio_max}")
    print(f"tracked_shell_file_count={tracked_shell_file_count}")
    print(f"tracked_rust_file_count={tracked_rust_file_count}")
    print(f"delta_to_warn_shell_rust_ratio_max={delta_to_warn_shell_rust_ratio_max}")
    print(f"delta_to_fail_shell_rust_ratio_max={delta_to_fail_shell_rust_ratio_max}")


def write_payload(payload: dict) -> bool:
    try:
        output_json.parent.mkdir(parents=True, exist_ok=True)
        output_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
        return True
    except Exception:
        return False


raw_thresholds: dict[str, str] = {}
for raw_line in threshold_file.read_text(encoding="utf-8", errors="ignore").splitlines():
    line = raw_line.strip()
    if not line or line.startswith("#"):
        continue
    if "=" not in line:
        payload = {
            "schema_version": "kamn.ci.shell-rust-ratio-guardrail-report.v1",
            "status": "fail",
            "final_decision": "NO-GO",
            "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
            "reason_codes": "shell_rust_ratio_threshold_value_invalid",
        }
        if not write_payload(payload):
            emit(
                "fail",
                "NO-GO",
                "shell_rust_ratio_output_write_failed",
                shell_line_total="unknown",
                rust_line_total="unknown",
                shell_to_rust_ratio="unknown",
                warn_shell_rust_ratio_max="unknown",
                fail_shell_rust_ratio_max="unknown",
                tracked_shell_file_count="unknown",
                tracked_rust_file_count="unknown",
                delta_to_warn_shell_rust_ratio_max="unknown",
                delta_to_fail_shell_rust_ratio_max="unknown",
            )
            raise SystemExit(1)
        emit(
            "fail",
            "NO-GO",
            "shell_rust_ratio_threshold_value_invalid",
            shell_line_total="unknown",
            rust_line_total="unknown",
            shell_to_rust_ratio="unknown",
            warn_shell_rust_ratio_max="unknown",
            fail_shell_rust_ratio_max="unknown",
            tracked_shell_file_count="unknown",
            tracked_rust_file_count="unknown",
            delta_to_warn_shell_rust_ratio_max="unknown",
            delta_to_fail_shell_rust_ratio_max="unknown",
        )
        raise SystemExit(1)
    key, value = line.split("=", 1)
    raw_thresholds[key.strip()] = value.strip()

warn_raw = raw_thresholds.get("WARN_SHELL_RUST_RATIO_MAX", "")
fail_raw = raw_thresholds.get("FAIL_SHELL_RUST_RATIO_MAX", "")
if not warn_raw or not fail_raw:
    payload = {
        "schema_version": "kamn.ci.shell-rust-ratio-guardrail-report.v1",
        "status": "fail",
        "final_decision": "NO-GO",
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": "shell_rust_ratio_threshold_key_missing",
    }
    if not write_payload(payload):
        emit(
            "fail",
            "NO-GO",
            "shell_rust_ratio_output_write_failed",
            shell_line_total="unknown",
            rust_line_total="unknown",
            shell_to_rust_ratio="unknown",
            warn_shell_rust_ratio_max="unknown",
            fail_shell_rust_ratio_max="unknown",
            tracked_shell_file_count="unknown",
            tracked_rust_file_count="unknown",
            delta_to_warn_shell_rust_ratio_max="unknown",
            delta_to_fail_shell_rust_ratio_max="unknown",
        )
        raise SystemExit(1)
    emit(
        "fail",
        "NO-GO",
        "shell_rust_ratio_threshold_key_missing",
        shell_line_total="unknown",
        rust_line_total="unknown",
        shell_to_rust_ratio="unknown",
        warn_shell_rust_ratio_max="unknown",
        fail_shell_rust_ratio_max="unknown",
        tracked_shell_file_count="unknown",
        tracked_rust_file_count="unknown",
        delta_to_warn_shell_rust_ratio_max="unknown",
        delta_to_fail_shell_rust_ratio_max="unknown",
    )
    raise SystemExit(1)

try:
    warn_max = float(warn_raw)
    fail_max = float(fail_raw)
except ValueError:
    payload = {
        "schema_version": "kamn.ci.shell-rust-ratio-guardrail-report.v1",
        "status": "fail",
        "final_decision": "NO-GO",
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": "shell_rust_ratio_threshold_value_invalid",
    }
    if not write_payload(payload):
        emit(
            "fail",
            "NO-GO",
            "shell_rust_ratio_output_write_failed",
            shell_line_total="unknown",
            rust_line_total="unknown",
            shell_to_rust_ratio="unknown",
            warn_shell_rust_ratio_max=warn_raw,
            fail_shell_rust_ratio_max=fail_raw,
            tracked_shell_file_count="unknown",
            tracked_rust_file_count="unknown",
            delta_to_warn_shell_rust_ratio_max="unknown",
            delta_to_fail_shell_rust_ratio_max="unknown",
        )
        raise SystemExit(1)
    emit(
        "fail",
        "NO-GO",
        "shell_rust_ratio_threshold_value_invalid",
        shell_line_total="unknown",
        rust_line_total="unknown",
        shell_to_rust_ratio="unknown",
        warn_shell_rust_ratio_max=warn_raw,
        fail_shell_rust_ratio_max=fail_raw,
        tracked_shell_file_count="unknown",
        tracked_rust_file_count="unknown",
        delta_to_warn_shell_rust_ratio_max="unknown",
        delta_to_fail_shell_rust_ratio_max="unknown",
    )
    raise SystemExit(1)

if warn_max >= fail_max:
    payload = {
        "schema_version": "kamn.ci.shell-rust-ratio-guardrail-report.v1",
        "status": "fail",
        "final_decision": "NO-GO",
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": "shell_rust_ratio_threshold_order_invalid",
        "metrics": {
            "warn_shell_rust_ratio_max": warn_max,
            "fail_shell_rust_ratio_max": fail_max,
        },
    }
    if not write_payload(payload):
        emit(
            "fail",
            "NO-GO",
            "shell_rust_ratio_output_write_failed",
            shell_line_total="unknown",
            rust_line_total="unknown",
            shell_to_rust_ratio="unknown",
            warn_shell_rust_ratio_max=str(warn_max),
            fail_shell_rust_ratio_max=str(fail_max),
            tracked_shell_file_count="unknown",
            tracked_rust_file_count="unknown",
            delta_to_warn_shell_rust_ratio_max="unknown",
            delta_to_fail_shell_rust_ratio_max="unknown",
        )
        raise SystemExit(1)
    emit(
        "fail",
        "NO-GO",
        "shell_rust_ratio_threshold_order_invalid",
        shell_line_total="unknown",
        rust_line_total="unknown",
        shell_to_rust_ratio="unknown",
        warn_shell_rust_ratio_max=str(warn_max),
        fail_shell_rust_ratio_max=str(fail_max),
        tracked_shell_file_count="unknown",
        tracked_rust_file_count="unknown",
        delta_to_warn_shell_rust_ratio_max="unknown",
        delta_to_fail_shell_rust_ratio_max="unknown",
    )
    raise SystemExit(1)

try:
    tracked_shell = subprocess.check_output(
        ["git", "-C", str(repo_root), "ls-files", "*.sh"],
        text=True,
    ).splitlines()
    tracked_rust = subprocess.check_output(
        ["git", "-C", str(repo_root), "ls-files", "*.rs"],
        text=True,
    ).splitlines()
except subprocess.CalledProcessError:
    payload = {
        "schema_version": "kamn.ci.shell-rust-ratio-guardrail-report.v1",
        "status": "fail",
        "final_decision": "NO-GO",
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": "shell_rust_ratio_git_ls_failed",
    }
    if not write_payload(payload):
        emit(
            "fail",
            "NO-GO",
            "shell_rust_ratio_output_write_failed",
            shell_line_total="unknown",
            rust_line_total="unknown",
            shell_to_rust_ratio="unknown",
            warn_shell_rust_ratio_max=str(warn_max),
            fail_shell_rust_ratio_max=str(fail_max),
            tracked_shell_file_count="unknown",
            tracked_rust_file_count="unknown",
            delta_to_warn_shell_rust_ratio_max="unknown",
            delta_to_fail_shell_rust_ratio_max="unknown",
        )
        raise SystemExit(1)
    emit(
        "fail",
        "NO-GO",
        "shell_rust_ratio_git_ls_failed",
        shell_line_total="unknown",
        rust_line_total="unknown",
        shell_to_rust_ratio="unknown",
        warn_shell_rust_ratio_max=str(warn_max),
        fail_shell_rust_ratio_max=str(fail_max),
        tracked_shell_file_count="unknown",
        tracked_rust_file_count="unknown",
        delta_to_warn_shell_rust_ratio_max="unknown",
        delta_to_fail_shell_rust_ratio_max="unknown",
    )
    raise SystemExit(1)

shell_files: list[Path] = []
for rel in tracked_shell:
    p = repo_root / rel
    if not p.is_file() or p.is_symlink():
        continue
    shell_files.append(p)

rust_files: list[Path] = []
for rel in tracked_rust:
    p = repo_root / rel
    if not p.is_file() or p.is_symlink():
        continue
    rust_files.append(p)

shell_line_total = 0
for path in shell_files:
    with path.open("r", encoding="utf-8", errors="ignore") as handle:
        shell_line_total += sum(1 for _ in handle)

rust_line_total = 0
for path in rust_files:
    with path.open("r", encoding="utf-8", errors="ignore") as handle:
        rust_line_total += sum(1 for _ in handle)

if rust_line_total <= 0:
    payload = {
        "schema_version": "kamn.ci.shell-rust-ratio-guardrail-report.v1",
        "status": "fail",
        "final_decision": "NO-GO",
        "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
        "reason_codes": "shell_rust_ratio_rust_line_total_invalid",
        "metrics": {
            "shell_line_total": shell_line_total,
            "rust_line_total": rust_line_total,
            "tracked_shell_file_count": len(shell_files),
            "tracked_rust_file_count": len(rust_files),
            "warn_shell_rust_ratio_max": warn_max,
            "fail_shell_rust_ratio_max": fail_max,
        },
    }
    if not write_payload(payload):
        emit(
            "fail",
            "NO-GO",
            "shell_rust_ratio_output_write_failed",
            shell_line_total=str(shell_line_total),
            rust_line_total=str(rust_line_total),
            shell_to_rust_ratio="unknown",
            warn_shell_rust_ratio_max=str(warn_max),
            fail_shell_rust_ratio_max=str(fail_max),
            tracked_shell_file_count=str(len(shell_files)),
            tracked_rust_file_count=str(len(rust_files)),
            delta_to_warn_shell_rust_ratio_max="unknown",
            delta_to_fail_shell_rust_ratio_max="unknown",
        )
        raise SystemExit(1)
    emit(
        "fail",
        "NO-GO",
        "shell_rust_ratio_rust_line_total_invalid",
        shell_line_total=str(shell_line_total),
        rust_line_total=str(rust_line_total),
        shell_to_rust_ratio="unknown",
        warn_shell_rust_ratio_max=str(warn_max),
        fail_shell_rust_ratio_max=str(fail_max),
        tracked_shell_file_count=str(len(shell_files)),
        tracked_rust_file_count=str(len(rust_files)),
        delta_to_warn_shell_rust_ratio_max="unknown",
        delta_to_fail_shell_rust_ratio_max="unknown",
    )
    raise SystemExit(1)

ratio = round(shell_line_total / rust_line_total, 6)
delta_to_warn = round(warn_max - ratio, 6)
delta_to_fail = round(fail_max - ratio, 6)

status = "ok"
final_decision = "GO"
reason_codes = "none"
if ratio > fail_max:
    status = "fail"
    final_decision = "NO-GO"
    reason_codes = "shell_rust_ratio_fail_threshold_exceeded"
elif ratio > warn_max:
    status = "ok"
    final_decision = "WARN"
    reason_codes = "shell_rust_ratio_warn_threshold_exceeded"

payload = {
    "schema_version": "kamn.ci.shell-rust-ratio-guardrail-report.v1",
    "status": status,
    "final_decision": final_decision,
    "reason_taxonomy_version": REASON_TAXONOMY_VERSION,
    "reason_codes": reason_codes,
    "metrics": {
        "shell_line_total": shell_line_total,
        "rust_line_total": rust_line_total,
        "shell_to_rust_ratio": ratio,
        "warn_shell_rust_ratio_max": warn_max,
        "fail_shell_rust_ratio_max": fail_max,
        "tracked_shell_file_count": len(shell_files),
        "tracked_rust_file_count": len(rust_files),
        "delta_to_warn_shell_rust_ratio_max": delta_to_warn,
        "delta_to_fail_shell_rust_ratio_max": delta_to_fail,
    },
}

if not write_payload(payload):
    emit(
        "fail",
        "NO-GO",
        "shell_rust_ratio_output_write_failed",
        shell_line_total=str(shell_line_total),
        rust_line_total=str(rust_line_total),
        shell_to_rust_ratio=str(ratio),
        warn_shell_rust_ratio_max=str(warn_max),
        fail_shell_rust_ratio_max=str(fail_max),
        tracked_shell_file_count=str(len(shell_files)),
        tracked_rust_file_count=str(len(rust_files)),
        delta_to_warn_shell_rust_ratio_max=str(delta_to_warn),
        delta_to_fail_shell_rust_ratio_max=str(delta_to_fail),
    )
    raise SystemExit(1)

emit(
    status,
    final_decision,
    reason_codes,
    shell_line_total=str(shell_line_total),
    rust_line_total=str(rust_line_total),
    shell_to_rust_ratio=str(ratio),
    warn_shell_rust_ratio_max=str(warn_max),
    fail_shell_rust_ratio_max=str(fail_max),
    tracked_shell_file_count=str(len(shell_files)),
    tracked_rust_file_count=str(len(rust_files)),
    delta_to_warn_shell_rust_ratio_max=str(delta_to_warn),
    delta_to_fail_shell_rust_ratio_max=str(delta_to_fail),
)

if status == "fail":
    raise SystemExit(1)
PY
)"
py_exit=$?
set -e

printf '%s\n' "$python_output"

if [[ "$py_exit" -ne 0 ]]; then
  exit 1
fi
