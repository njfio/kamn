#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
PARSER_SCRIPT="$ROOT_DIR/scripts/ci/ignored_test_inventory.py"
BASELINE_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_baseline.json"
METADATA_FILE="$ROOT_DIR/fixtures/ci/ignored_test_inventory_metadata.json"
PROMOTION_CRITERIA_FILE="$ROOT_DIR/fixtures/ci/ignored_test_promotion_criteria.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

if [ ! -x "$PARSER_SCRIPT" ]; then
  echo "expected ignored-test inventory parser script to be executable" >&2
  exit 1
fi

if [ ! -f "$BASELINE_FILE" ]; then
  echo "expected ignored-test baseline fixture to exist" >&2
  exit 1
fi

if [ ! -f "$METADATA_FILE" ]; then
  echo "expected ignored-test metadata fixture to exist" >&2
  exit 1
fi

if [ ! -f "$PROMOTION_CRITERIA_FILE" ]; then
  echo "expected ignored-test promotion criteria fixture to exist" >&2
  exit 1
fi

SAMPLE_REPO="$TMP_DIR/sample-repo"
mkdir -p "$SAMPLE_REPO/crates/sample/src"
cat >"$SAMPLE_REPO/crates/sample/src/parser_contract.rs" <<'RS'
#[cfg(test)]
mod tests {
    #[test]
    #[ignore]
    fn ignored_alpha() {
        assert!(true);
    }

    #[test]
    #[ignore = "deep lane"]
    async fn ignored_beta() {
        assert!(true);
    }
}
RS

cat >"$SAMPLE_REPO/crates/sample/src/unresolved_marker.rs" <<'RS'
#[ignore]
const NOT_A_TEST: usize = 1;
RS

python3 - "$ROOT_DIR" "$SAMPLE_REPO" "$BASELINE_FILE" "$METADATA_FILE" "$PROMOTION_CRITERIA_FILE" <<'PY'
from __future__ import annotations

import importlib.util
import json
import pathlib
import sys

root_dir = pathlib.Path(sys.argv[1])
sample_repo = pathlib.Path(sys.argv[2])
baseline_file = pathlib.Path(sys.argv[3])
metadata_file = pathlib.Path(sys.argv[4])
promotion_criteria_file = pathlib.Path(sys.argv[5])

module_path = root_dir / "scripts/ci/ignored_test_inventory.py"
spec = importlib.util.spec_from_file_location("ignored_test_inventory", module_path)
if spec is None or spec.loader is None:
    raise SystemExit("failed to load ignored_test_inventory module")
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)

ignored_tests, unresolved_markers = module.collect_ignored_tests(
    repo_root=sample_repo,
    scan_roots=["crates"],
)
expected_ignored_tests = [
    {
        "source_file": "crates/sample/src/parser_contract.rs",
        "test_name": "ignored_alpha",
    },
    {
        "source_file": "crates/sample/src/parser_contract.rs",
        "test_name": "ignored_beta",
    },
]
if ignored_tests != expected_ignored_tests:
    raise SystemExit(
        f"unexpected ignored-test parser extraction: expected {expected_ignored_tests}, got {ignored_tests}"
    )
if len(unresolved_markers) != 1:
    raise SystemExit(
        f"expected one unresolved ignore marker from parser contract sample, found {len(unresolved_markers)}"
    )
if unresolved_markers[0] != "crates/sample/src/unresolved_marker.rs:1:ignore_attribute_without_function":
    raise SystemExit(
        "unexpected unresolved marker emitted by parser contract sample: "
        f"{unresolved_markers[0]}"
    )

baseline_payload = json.loads(baseline_file.read_text(encoding="utf-8"))
validated_inventory = module.validate_inventory_payload(
    baseline_payload,
    label="baseline fixture",
)
if len(validated_inventory) != baseline_payload["ignored_test_count"]:
    raise SystemExit("baseline fixture ignored_test_count mismatch in parser contract check")
if validated_inventory != sorted(
    validated_inventory,
    key=lambda item: (item["source_file"], item["test_name"]),
):
    raise SystemExit("baseline fixture ignored_tests should remain deterministically sorted")

metadata_payload = json.loads(metadata_file.read_text(encoding="utf-8"))
metadata_by_key = module.validate_metadata_payload(
    metadata_payload,
    label="metadata fixture",
)
if len(metadata_by_key) != baseline_payload["ignored_test_count"]:
    raise SystemExit("expected metadata fixture coverage for all baseline ignored tests")

for entry in validated_inventory:
    key = (entry["source_file"], entry["test_name"])
    metadata = metadata_by_key.get(key)
    if metadata is None:
        raise SystemExit(f"missing metadata entry for baseline ignored test: {key[0]}::{key[1]}")
    if not metadata["reason"]:
        raise SystemExit(f"metadata reason missing for ignored test: {key[0]}::{key[1]}")

promotion_criteria_payload = json.loads(promotion_criteria_file.read_text(encoding="utf-8"))
criteria_by_category = module.validate_promotion_criteria_payload(
    promotion_criteria_payload,
    label="promotion criteria fixture",
)

for key, metadata in metadata_by_key.items():
    reason_category = metadata["reason"]
    if reason_category not in criteria_by_category:
        raise SystemExit(
            f"missing promotion criteria category for ignored-test reason {reason_category}: "
            f"{key[0]}::{key[1]}"
        )

PY

echo "ignored-test inventory parser contract tests passed."
