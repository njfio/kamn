#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

output_json=""
max_seconds=180

while [[ $# -gt 0 ]]; do
  case "$1" in
    --output-json)
      output_json="${2:-}"
      shift 2
      ;;
    --max-seconds)
      max_seconds="${2:-}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 1
      ;;
  esac
done

if ! [[ "$max_seconds" =~ ^[0-9]+$ ]]; then
  echo "max-seconds must be an integer" >&2
  exit 1
fi

TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

start_epoch="$(date +%s)"

pushd "$ROOT_DIR" >/dev/null
cargo test -p kamn-core --test content_storage_file_adapter -- --nocapture \
  >"$TMP_DIR/content-storage-file-adapter.log" 2>&1
cargo test -p kamn-core --test did_registry_file_chain_adapter -- --nocapture \
  >"$TMP_DIR/did-registry-file-adapter.log" 2>&1
popd >/dev/null

elapsed_seconds="$(( $(date +%s) - start_epoch ))"
if [ "$elapsed_seconds" -gt "$max_seconds" ]; then
  echo "persistence adapter live validation exceeded runtime budget: ${elapsed_seconds}s" >&2
  exit 1
fi

report_json="$TMP_DIR/persistence-adapter-live-validation-report.json"
cat >"$report_json" <<JSON
{
  "schema_version": "kamn.persistence.adapters-live-validation.v1",
  "status": "pass",
  "final_decision": "GO",
  "content_persistence_status": "verified",
  "did_duplicate_detection_status": "verified",
  "fail_closed_status": "verified",
  "elapsed_seconds": $elapsed_seconds
}
JSON

if [[ -n "$output_json" ]]; then
  cp "$report_json" "$output_json"
fi

echo "status=pass"
echo "final_decision=GO"
echo "content_persistence_status=verified"
echo "did_duplicate_detection_status=verified"
echo "fail_closed_status=verified"
