#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
FAST_WORKFLOW="$ROOT_DIR/.github/workflows/ci-fast-gate.yml"
CI_TOOLS_SCRIPT="$ROOT_DIR/scripts/ci/test_ci_tools.sh"
POLICY_FILE="$ROOT_DIR/.ci/kolme-command-surface-asymmetry-policy.json"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_file "$FAST_WORKFLOW" "expected fast-gate workflow to exist"

test_harness_require_file "$CI_TOOLS_SCRIPT" "expected aggregate CI tools script to exist"

test_harness_require_file "$POLICY_FILE" "expected Kolme asymmetry policy file to exist at .ci/kolme-command-surface-asymmetry-policy.json"

mapfile -t kolme_tests < <(find "$ROOT_DIR/scripts/kolme" -maxdepth 1 -type f -name 'test_*.sh' | sort)
if [ "${#kolme_tests[@]}" -eq 0 ]; then
  echo "expected Kolme command-surface tests under scripts/kolme" >&2
  exit 1
fi

actual_fast_only=()
actual_ci_tools_only=()

for script_path in "${kolme_tests[@]}"; do
  relative_path="${script_path#"$ROOT_DIR/"}"
  in_fast=false
  in_ci_tools=false

  if grep -Fq "$relative_path" "$FAST_WORKFLOW"; then
    in_fast=true
  fi
  if grep -Fq "$relative_path" "$CI_TOOLS_SCRIPT"; then
    in_ci_tools=true
  fi

  if [ "$in_fast" = true ] && [ "$in_ci_tools" = false ]; then
    actual_fast_only+=("$relative_path")
  elif [ "$in_fast" = false ] && [ "$in_ci_tools" = true ]; then
    actual_ci_tools_only+=("$relative_path")
  fi
done

printf '%s\n' "${actual_fast_only[@]}" | sed '/^$/d' | sort >"$TMP_DIR/actual_fast_only.txt"
printf '%s\n' "${actual_ci_tools_only[@]}" | sed '/^$/d' | sort >"$TMP_DIR/actual_ci_tools_only.txt"

python3 - "$POLICY_FILE" "$TMP_DIR/expected_fast_only.txt" "$TMP_DIR/expected_ci_tools_only.txt" <<'PY'
import json
import pathlib
import sys

policy_path = pathlib.Path(sys.argv[1])
expected_fast_only_path = pathlib.Path(sys.argv[2])
expected_ci_tools_only_path = pathlib.Path(sys.argv[3])

try:
    policy = json.loads(policy_path.read_text(encoding="utf-8"))
except (OSError, json.JSONDecodeError) as exc:
    raise SystemExit(f"invalid Kolme asymmetry policy file: {exc}")

if policy.get("schema_version") != "kamn.ci.kolme-command-surface-asymmetry-policy.v1":
    raise SystemExit("unexpected Kolme asymmetry policy schema_version")

def validate_array(name: str) -> list[str]:
    value = policy.get(name)
    if not isinstance(value, list) or len(value) == 0:
        raise SystemExit(f"expected non-empty '{name}' array in Kolme asymmetry policy")
    if not all(isinstance(item, str) and item for item in value):
        raise SystemExit(f"expected '{name}' entries to be non-empty strings")
    if len(set(value)) != len(value):
        raise SystemExit(f"expected '{name}' entries to be unique")
    return sorted(value)

expected_fast_only = validate_array("fast_only")
expected_ci_tools_only = validate_array("ci_tools_only")

expected_fast_only_path.write_text("".join(f"{item}\n" for item in expected_fast_only), encoding="utf-8")
expected_ci_tools_only_path.write_text("".join(f"{item}\n" for item in expected_ci_tools_only), encoding="utf-8")
PY

echo "kolme_fast_only_count_actual=$(wc -l <"$TMP_DIR/actual_fast_only.txt" | tr -d ' ')"
echo "kolme_fast_only_count_expected=$(wc -l <"$TMP_DIR/expected_fast_only.txt" | tr -d ' ')"
echo "kolme_ci_tools_only_count_actual=$(wc -l <"$TMP_DIR/actual_ci_tools_only.txt" | tr -d ' ')"
echo "kolme_ci_tools_only_count_expected=$(wc -l <"$TMP_DIR/expected_ci_tools_only.txt" | tr -d ' ')"

if ! diff -u "$TMP_DIR/expected_fast_only.txt" "$TMP_DIR/actual_fast_only.txt" >/dev/null; then
  echo "expected fast-only Kolme command-surface set to match approved policy" >&2
  diff -u "$TMP_DIR/expected_fast_only.txt" "$TMP_DIR/actual_fast_only.txt" >&2
  exit 1
fi

if ! diff -u "$TMP_DIR/expected_ci_tools_only.txt" "$TMP_DIR/actual_ci_tools_only.txt" >/dev/null; then
  echo "expected ci-tools-only Kolme command-surface set to match approved policy" >&2
  diff -u "$TMP_DIR/expected_ci_tools_only.txt" "$TMP_DIR/actual_ci_tools_only.txt" >&2
  exit 1
fi

echo "kolme command-surface asymmetry contract tests passed."
