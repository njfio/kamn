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

provenance = payload.get("baseline_provenance")
if not isinstance(provenance, dict):
    raise SystemExit("fixture matrix must include baseline_provenance object")

provenance_required_fields = (
    "artifact_version",
    "source_commit",
    "source_run_id",
    "generated_at_utc",
    "generator",
)
for field in provenance_required_fields:
    value = provenance.get(field)
    if not isinstance(value, str) or not value.strip():
        raise SystemExit(f"baseline_provenance.{field} must be non-empty string")

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

drift_seed_id = matching.get("drift_threshold_seed_id")
if not isinstance(drift_seed_id, str) or not drift_seed_id.strip():
    raise SystemExit("fixture field drift_threshold_seed_id must be non-empty string")

drift_seed = matching.get("drift_threshold_seed")
if not isinstance(drift_seed, dict):
    raise SystemExit("fixture field drift_threshold_seed must be an object")

drift_seed_required_fields = (
    "max_latency_p50_ms",
    "max_latency_p99_ms",
    "min_throughput_tps",
    "min_availability_pct",
)
for field in drift_seed_required_fields:
    value = drift_seed.get(field)
    if not isinstance(value, (int, float)):
        raise SystemExit(f"drift_threshold_seed.{field} must be numeric")

if drift_seed["max_latency_p50_ms"] <= 0 or drift_seed["max_latency_p99_ms"] <= 0:
    raise SystemExit("drift threshold max latency values must be > 0")
if drift_seed["min_throughput_tps"] <= 0:
    raise SystemExit("drift threshold min throughput must be > 0")
if drift_seed["min_availability_pct"] <= 0 or drift_seed["min_availability_pct"] > 100:
    raise SystemExit("drift threshold min availability must be within (0, 100]")

profile = matching.get("profile")
if not isinstance(profile, str) or not profile.strip():
    profile = f"prd-13.2-ci-{workload}-{lane}"

print(f"PROFILE={profile}")
print(f"WORKLOAD={workload}")
print(f"LATENCY_P50_MS={matching['latency_p50_ms']}")
print(f"LATENCY_P99_MS={matching['latency_p99_ms']}")
print(f"THROUGHPUT_TPS={matching['throughput_tps']}")
print(f"AVAILABILITY_PCT={matching['availability_pct']}")
print(f"BASELINE_PROVENANCE_ARTIFACT_VERSION={provenance['artifact_version']}")
print(f"BASELINE_PROVENANCE_SOURCE_COMMIT={provenance['source_commit']}")
print(f"BASELINE_PROVENANCE_SOURCE_RUN_ID={provenance['source_run_id']}")
print(f"BASELINE_PROVENANCE_GENERATED_AT_UTC={provenance['generated_at_utc']}")
print(f"BASELINE_PROVENANCE_GENERATOR={provenance['generator']}")
print(f"DRIFT_THRESHOLD_SEED_ID={drift_seed_id}")
print(f"DRIFT_THRESHOLD_SEED_MAX_LATENCY_P50_MS={drift_seed['max_latency_p50_ms']}")
print(f"DRIFT_THRESHOLD_SEED_MAX_LATENCY_P99_MS={drift_seed['max_latency_p99_ms']}")
print(f"DRIFT_THRESHOLD_SEED_MIN_THROUGHPUT_TPS={drift_seed['min_throughput_tps']}")
print(f"DRIFT_THRESHOLD_SEED_MIN_AVAILABILITY_PCT={drift_seed['min_availability_pct']}")
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
  "availability_pct": $AVAILABILITY_PCT,
  "baseline_provenance_artifact_version": "$BASELINE_PROVENANCE_ARTIFACT_VERSION",
  "baseline_provenance_source_commit": "$BASELINE_PROVENANCE_SOURCE_COMMIT",
  "baseline_provenance_source_run_id": "$BASELINE_PROVENANCE_SOURCE_RUN_ID",
  "baseline_provenance_generated_at_utc": "$BASELINE_PROVENANCE_GENERATED_AT_UTC",
  "baseline_provenance_generator": "$BASELINE_PROVENANCE_GENERATOR",
  "drift_threshold_seed_id": "$DRIFT_THRESHOLD_SEED_ID",
  "drift_threshold_seed_max_latency_p50_ms": $DRIFT_THRESHOLD_SEED_MAX_LATENCY_P50_MS,
  "drift_threshold_seed_max_latency_p99_ms": $DRIFT_THRESHOLD_SEED_MAX_LATENCY_P99_MS,
  "drift_threshold_seed_min_throughput_tps": $DRIFT_THRESHOLD_SEED_MIN_THROUGHPUT_TPS,
  "drift_threshold_seed_min_availability_pct": $DRIFT_THRESHOLD_SEED_MIN_AVAILABILITY_PCT
}
JSON

echo "generated performance report: workload=${WORKLOAD}; lane=${LANE}; output=${OUTPUT_JSON}"
