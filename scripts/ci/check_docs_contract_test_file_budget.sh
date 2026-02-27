#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DEFAULT_THRESHOLD_FILE="$ROOT_DIR/.ci/docs-contract-test-file-budget.env"
REASON_TAXONOMY_VERSION="kamn.ci.docs-contract-test-file-budget-reason-taxonomy.v1"
REASON_CODES_CSV="docs_contract_test_file_budget_argument_invalid,docs_contract_test_file_budget_threshold_file_missing,docs_contract_test_file_budget_threshold_key_missing,docs_contract_test_file_budget_threshold_value_invalid,docs_contract_test_file_budget_exceeded,docs_contract_test_file_budget_git_ls_failed,docs_contract_test_file_budget_output_json_required,docs_contract_test_file_budget_output_write_failed"

repo_root="$ROOT_DIR"
threshold_file="$DEFAULT_THRESHOLD_FILE"
output_json=""

emit_result() {
  local status="$1"
  local final_decision="$2"
  local reason_codes="$3"
  local docs_contract_test_file_count="$4"
  local docs_contract_test_file_max="$5"
  local delta_to_docs_contract_test_file_max="$6"

  echo "status=$status"
  echo "final_decision=$final_decision"
  echo "reason_taxonomy_version=$REASON_TAXONOMY_VERSION"
  echo "reason_codes=$reason_codes"
  echo "reason_codes_csv=$REASON_CODES_CSV"
  echo "docs_contract_test_file_count=$docs_contract_test_file_count"
  echo "docs_contract_test_file_max=$docs_contract_test_file_max"
  echo "delta_to_docs_contract_test_file_max=$delta_to_docs_contract_test_file_max"
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
      emit_result "fail" "NO-GO" "docs_contract_test_file_budget_argument_invalid" "unknown" "unknown" "unknown"
      exit 1
      ;;
  esac
done

if [[ -z "$output_json" ]]; then
  emit_result "fail" "NO-GO" "docs_contract_test_file_budget_output_json_required" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -d "$repo_root" ]]; then
  emit_result "fail" "NO-GO" "docs_contract_test_file_budget_argument_invalid" "unknown" "unknown" "unknown"
  exit 1
fi

if [[ ! -f "$threshold_file" ]]; then
  emit_result "fail" "NO-GO" "docs_contract_test_file_budget_threshold_file_missing" "unknown" "unknown" "unknown"
  exit 1
fi

docs_contract_test_file_max_raw="$(grep -E '^DOCS_CONTRACT_TEST_FILE_MAX=' "$threshold_file" | head -n 1 | cut -d= -f2- || true)"
if [[ -z "$docs_contract_test_file_max_raw" ]]; then
  emit_result "fail" "NO-GO" "docs_contract_test_file_budget_threshold_key_missing" "unknown" "unknown" "unknown"
  exit 1
fi
if [[ ! "$docs_contract_test_file_max_raw" =~ ^[0-9]+$ ]]; then
  emit_result "fail" "NO-GO" "docs_contract_test_file_budget_threshold_value_invalid" "unknown" "$docs_contract_test_file_max_raw" "unknown"
  exit 1
fi

set +e
tracked_docs_tests="$(git -C "$repo_root" ls-files 'crates/*/tests/*_docs.rs')"
git_ls_exit=$?
set -e
if [[ "$git_ls_exit" -ne 0 ]]; then
  emit_result "fail" "NO-GO" "docs_contract_test_file_budget_git_ls_failed" "unknown" "$docs_contract_test_file_max_raw" "unknown"
  exit 1
fi
docs_contract_test_file_count=0
while IFS= read -r rel_path; do
  if [[ -z "$rel_path" ]]; then
    continue
  fi
  if [[ -f "$repo_root/$rel_path" ]]; then
    docs_contract_test_file_count=$((docs_contract_test_file_count + 1))
  fi
done <<<"$tracked_docs_tests"
if [[ ! "$docs_contract_test_file_count" =~ ^[0-9]+$ ]]; then
  emit_result "fail" "NO-GO" "docs_contract_test_file_budget_git_ls_failed" "$docs_contract_test_file_count" "$docs_contract_test_file_max_raw" "unknown"
  exit 1
fi

docs_contract_test_file_max="$docs_contract_test_file_max_raw"
delta_to_docs_contract_test_file_max="$((docs_contract_test_file_max - docs_contract_test_file_count))"

status="ok"
final_decision="GO"
reason_codes="none"
if [[ "$docs_contract_test_file_count" -gt "$docs_contract_test_file_max" ]]; then
  status="fail"
  final_decision="NO-GO"
  reason_codes="docs_contract_test_file_budget_exceeded"
fi

tmp_json="$(mktemp)"
trap 'rm -f "$tmp_json"' EXIT
bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$tmp_json" <<JSON
{
  "schema_version": "kamn.ci.docs-contract-test-file-budget-report.v1",
  "status": "$status",
  "final_decision": "$final_decision",
  "reason_taxonomy_version": "$REASON_TAXONOMY_VERSION",
  "reason_codes": "$reason_codes",
  "metrics": {
    "docs_contract_test_file_count": $docs_contract_test_file_count,
    "docs_contract_test_file_max": $docs_contract_test_file_max,
    "delta_to_docs_contract_test_file_max": $delta_to_docs_contract_test_file_max
  }
}
JSON

mkdir -p "$(dirname "$output_json")"
if ! cp "$tmp_json" "$output_json"; then
  emit_result "fail" "NO-GO" "docs_contract_test_file_budget_output_write_failed" "$docs_contract_test_file_count" "$docs_contract_test_file_max" "$delta_to_docs_contract_test_file_max"
  exit 1
fi

emit_result "$status" "$final_decision" "$reason_codes" "$docs_contract_test_file_count" "$docs_contract_test_file_max" "$delta_to_docs_contract_test_file_max"

if [[ "$status" != "ok" ]]; then
  exit 1
fi
