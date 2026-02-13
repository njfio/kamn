#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'USAGE'
Usage: run_kamn_core_rustdoc_artifact_contract_lane.sh \
  --output-json <path> \
  [--artifact-dir <dir>] \
  [--max-runtime-seconds <int>]
USAGE
}

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT_DIR"

OUTPUT_JSON=""
ARTIFACT_DIR="${TMPDIR:-/tmp}"
MAX_RUNTIME_SECONDS="${KAMN_CORE_RUSTDOC_ARTIFACT_MAX_SECONDS:-180}"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      OUTPUT_JSON="${2:-}"
      shift 2
      ;;
    --artifact-dir)
      ARTIFACT_DIR="${2:-}"
      shift 2
      ;;
    --max-runtime-seconds)
      MAX_RUNTIME_SECONDS="${2:-}"
      shift 2
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "Unknown argument: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
done

if [ -z "$OUTPUT_JSON" ]; then
  usage >&2
  exit 2
fi

case "$MAX_RUNTIME_SECONDS" in
  ''|*[!0-9]*)
    echo "--max-runtime-seconds must be a non-negative integer" >&2
    exit 2
    ;;
esac

mkdir -p "$ARTIFACT_DIR"
mkdir -p "$(dirname "$OUTPUT_JSON")"

RUSTDOC_COMMAND_DESCRIPTOR='RUSTDOCFLAGS=-D warnings cargo doc -p kamn-core --no-deps'
start_epoch="$(date +%s)"
RUSTDOCFLAGS="-D warnings" cargo doc -p kamn-core --no-deps >/dev/null

artifact_path="$ARTIFACT_DIR/kamn-core-rustdoc.tar.gz"
tar -czf "$artifact_path" -C target doc

artifact_bytes="$(wc -c <"$artifact_path" | tr -d '[:space:]')"
artifact_sha256="$(sha256sum "$artifact_path" | awk '{print $1}')"
elapsed_seconds="$(( $(date +%s) - start_epoch ))"

status="pass"
reason_key="kamn.ci.kamn-core-rustdoc-artifact.ok"
if [ "$elapsed_seconds" -gt "$MAX_RUNTIME_SECONDS" ]; then
  status="fail"
  reason_key="kamn.ci.kamn-core-rustdoc-artifact.runtime-budget-exceeded"
fi

cat >"$OUTPUT_JSON" <<JSON
{
  "schema_version": "kamn.ci.kamn-core-rustdoc-artifact-report.v1",
  "status": "$status",
  "crate": "kamn-core",
  "command": "$RUSTDOC_COMMAND_DESCRIPTOR",
  "artifact_path": "$artifact_path",
  "artifact_bytes": $artifact_bytes,
  "artifact_sha256": "$artifact_sha256",
  "runtime_seconds": $elapsed_seconds,
  "max_runtime_seconds": $MAX_RUNTIME_SECONDS,
  "reason_key": "$reason_key"
}
JSON

echo "kamn_core_rustdoc_artifact_status=$status"
echo "kamn_core_rustdoc_artifact_report=$OUTPUT_JSON"

if [ "$status" != "pass" ]; then
  echo "rustdoc artifact contract lane exceeded runtime budget: ${elapsed_seconds}s > ${MAX_RUNTIME_SECONDS}s" >&2
  exit 1
fi

echo "kamn-core rustdoc artifact contract lane tests passed."
