#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"

usage() {
  cat <<USAGE
Usage: $0 --output-json <path> [--lane <smoke|deep>] [--workload <runtime|signing|transport>] [--fixture-file <path>]
USAGE
}

OUTPUT_JSON=""
LANE="smoke"
WORKLOAD="runtime"
FIXTURE_FILE="$ROOT_DIR/fixtures/ci/performance_hot_path_fixture_matrix.json"
FIXTURE_SCHEMA_VERSION="kamn.ci.performance-hot-path-matrix.v1"

while [ "$#" -gt 0 ]; do
  case "$1" in
    --output-json)
      OUTPUT_JSON="${2:-}"
      shift 2
      ;;
    --lane)
      LANE="${2:-smoke}"
      shift 2
      ;;
    --workload)
      WORKLOAD="${2:-runtime}"
      shift 2
      ;;
    --fixture-file)
      FIXTURE_FILE="${2:-}"
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

if [ ! -f "$FIXTURE_FILE" ]; then
  echo "fixture file not found: $FIXTURE_FILE" >&2
  exit 2
fi

LOOKUP_OUTPUT="$(python3 - "$FIXTURE_FILE" "$FIXTURE_SCHEMA_VERSION" "$WORKLOAD" "$LANE" <<'PY'
import json
import sys
from pathlib import Path

fixture_path = Path(sys.argv[1])
expected_schema = sys.argv[2]
workload = sys.argv[3]
lane = sys.argv[4]

try:
    payload = json.loads(fixture_path.read_text(encoding="utf-8"))
except Exception as exc:
    raise SystemExit(f"failed to parse fixture matrix: {exc}")

schema_version = payload.get("schema_version")
if schema_version != expected_schema:
    raise SystemExit(
        "fixture schema version mismatch: "
        f"expected {expected_schema}, got {schema_version}"
    )

fixtures = payload.get("fixtures")
if not isinstance(fixtures, list) or not fixtures:
    raise SystemExit("fixture matrix must include non-empty fixtures array")

known_workloads = sorted(
    {
        item.get("workload")
        for item in fixtures
        if isinstance(item, dict) and isinstance(item.get("workload"), str)
    }
)

if workload not in known_workloads:
    raise SystemExit(f"Unknown workload: {workload}")

matching = None
for item in fixtures:
    if not isinstance(item, dict):
        continue
    if item.get("workload") == workload and item.get("lane") == lane:
        matching = item
        break

if matching is None:
    raise SystemExit(f"Unsupported lane for workload {workload}: {lane}")

required_fields = (
    "latency_p50_ms",
    "latency_p99_ms",
    "throughput_tps",
    "availability_pct",
)
for field in required_fields:
    value = matching.get(field)
    if not isinstance(value, (int, float)):
        raise SystemExit(f"fixture field {field} must be numeric")

if matching["latency_p50_ms"] < 0 or matching["latency_p99_ms"] < 0:
    raise SystemExit("latency fields must be non-negative")
if matching["throughput_tps"] <= 0:
    raise SystemExit("throughput_tps must be > 0")
if matching["availability_pct"] <= 0 or matching["availability_pct"] > 100:
    raise SystemExit("availability_pct must be within (0, 100]")

profile = matching.get("profile")
if not isinstance(profile, str) or not profile.strip():
    profile = f"prd-13.2-ci-{workload}-{lane}"

print(f"PROFILE={profile}")
print(f"WORKLOAD={workload}")
print(f"LATENCY_P50_MS={matching['latency_p50_ms']}")
print(f"LATENCY_P99_MS={matching['latency_p99_ms']}")
print(f"THROUGHPUT_TPS={matching['throughput_tps']}")
print(f"AVAILABILITY_PCT={matching['availability_pct']}")
PY
)" || exit $?

eval "$LOOKUP_OUTPUT"

bash "$ROOT_DIR/scripts/lib/write_json_file.sh" "$OUTPUT_JSON" <<JSON
{
  "profile": "$PROFILE",
  "lane": "$LANE",
  "workload": "$WORKLOAD",
  "latency_p50_ms": $LATENCY_P50_MS,
  "latency_p99_ms": $LATENCY_P99_MS,
  "throughput_tps": $THROUGHPUT_TPS,
  "availability_pct": $AVAILABILITY_PCT
}
JSON

echo "generated performance report: workload=${WORKLOAD}; lane=${LANE}; output=${OUTPUT_JSON}"
