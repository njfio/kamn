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
DOCS_CONTRACT_TEST_COUNT="${KAMN_RUSTDOC_NAV_DOCS_CONTRACT_TEST_COUNT:-2}"
BEHAVIORAL_TEST_COUNT="${KAMN_RUSTDOC_NAV_BEHAVIORAL_TEST_COUNT:-2}"
MAX_DOCS_CONTRACT_TO_BEHAVIORAL_RATIO="${KAMN_RUSTDOC_NAV_MAX_DOCS_TO_BEHAVIORAL_RATIO:-1.0}"

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

case "$DOCS_CONTRACT_TEST_COUNT" in
  ''|*[!0-9]*)
    echo "docs contract test count must be a non-negative integer" >&2
    exit 2
    ;;
esac

case "$BEHAVIORAL_TEST_COUNT" in
  ''|*[!0-9]*)
    echo "behavioral test count must be a positive integer" >&2
    exit 2
    ;;
esac
if [ "$BEHAVIORAL_TEST_COUNT" -le 0 ]; then
  echo "behavioral test count must be greater than zero" >&2
  exit 2
fi

if [[ ! "$MAX_DOCS_CONTRACT_TO_BEHAVIORAL_RATIO" =~ ^[0-9]+([.][0-9]+)?$ ]]; then
  echo "max docs-to-behavioral ratio must be a non-negative number" >&2
  exit 2
fi

docs_behavioral_ratio="$(
  python3 - "$DOCS_CONTRACT_TEST_COUNT" "$BEHAVIORAL_TEST_COUNT" <<'PY'
import sys

docs = int(sys.argv[1])
behavioral = int(sys.argv[2])
ratio = docs / behavioral
print(f"{ratio:.4f}")
PY
)"

rustdoc_navigation_ratio_status="$(
  python3 - "$docs_behavioral_ratio" "$MAX_DOCS_CONTRACT_TO_BEHAVIORAL_RATIO" <<'PY'
import sys

observed = float(sys.argv[1])
maximum = float(sys.argv[2])
print("exceeded" if observed > maximum else "within")
PY
)"

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
if [ "$rustdoc_navigation_ratio_status" = "exceeded" ]; then
  status="fail"
  reason_key="kamn.ci.kamn-core-rustdoc-artifact.docs-behavioral-ratio-threshold-exceeded"
fi

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$OUTPUT_JSON" <<JSON
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
  "docs_contract_test_count": $DOCS_CONTRACT_TEST_COUNT,
  "behavioral_test_count": $BEHAVIORAL_TEST_COUNT,
  "docs_contract_to_behavioral_ratio": $docs_behavioral_ratio,
  "max_docs_contract_to_behavioral_ratio": $MAX_DOCS_CONTRACT_TO_BEHAVIORAL_RATIO,
  "rustdoc_navigation_ratio_status": "$rustdoc_navigation_ratio_status",
  "reason_key": "$reason_key"
}
JSON

echo "kamn_core_rustdoc_artifact_status=$status"
echo "kamn_core_rustdoc_artifact_report=$OUTPUT_JSON"
echo "rustdoc_navigation_ratio_status=$rustdoc_navigation_ratio_status"
echo "docs_contract_test_count=$DOCS_CONTRACT_TEST_COUNT"
echo "behavioral_test_count=$BEHAVIORAL_TEST_COUNT"
echo "docs_contract_to_behavioral_ratio=$docs_behavioral_ratio"
echo "max_docs_contract_to_behavioral_ratio=$MAX_DOCS_CONTRACT_TO_BEHAVIORAL_RATIO"

if [ "$status" != "pass" ]; then
  if [ "$reason_key" = "kamn.ci.kamn-core-rustdoc-artifact.runtime-budget-exceeded" ]; then
    echo "rustdoc artifact contract lane exceeded runtime budget: ${elapsed_seconds}s > ${MAX_RUNTIME_SECONDS}s" >&2
  elif [ "$reason_key" = "kamn.ci.kamn-core-rustdoc-artifact.docs-behavioral-ratio-threshold-exceeded" ]; then
    echo "rustdoc navigation docs-vs-behavioral ratio exceeded threshold: ${docs_behavioral_ratio} > ${MAX_DOCS_CONTRACT_TO_BEHAVIORAL_RATIO}" >&2
  fi
  exit 1
fi

echo "kamn-core rustdoc artifact contract lane tests passed."
