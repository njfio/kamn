#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
source "$ROOT_DIR/scripts/lib/test_harness.sh"
CHECKER="$ROOT_DIR/scripts/ci/check_workspace_license_policy.py"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

test_harness_require_executable "$CHECKER" "expected workspace license policy checker to be executable"

pass_output="$(
  python3 "$CHECKER" \
    --workspace-root "$ROOT_DIR" \
    --expected-license "Apache-2.0" \
    --license-policy-file "$ROOT_DIR/LICENSE"
)"
if ! printf '%s\n' "$pass_output" | grep -q '^reason_taxonomy_version=kamn.ci.dependency-license-metadata-governance-reason-taxonomy.v1$'; then
  echo "expected deterministic reason taxonomy marker on pass output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_csv=none$'; then
  echo "expected deterministic reason-codes csv marker on pass output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_codes_value=none$'; then
  echo "expected deterministic reason-codes value marker on pass output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^reason_class=stable$'; then
  echo "expected deterministic reason-class marker on pass output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^ci_smoke_local_heavy_boundary_status=verified$'; then
  echo "expected ci/local boundary status marker on pass output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^ci_smoke_lane_cost_profile=low$'; then
  echo "expected ci smoke cost-profile marker on pass output" >&2
  exit 1
fi
if ! printf '%s\n' "$pass_output" | grep -q '^local_heavy_lane_execution_mode=not_requested$'; then
  echo "expected local-heavy execution-mode marker on pass output" >&2
  exit 1
fi

ROOT_POLICY_MISMATCH_LICENSE="$TMP_DIR/LICENSE.mismatch"
cp "$ROOT_DIR/LICENSE" "$ROOT_POLICY_MISMATCH_LICENSE"
python3 - "$ROOT_POLICY_MISMATCH_LICENSE" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(r"Version 2\.0, January 2004", "Version 1.0, January 2000", text, count=1)
path.write_text(text, encoding="utf-8")
PY

if python3 "$CHECKER" \
  --workspace-root "$ROOT_DIR" \
  --expected-license "Apache-2.0" \
  --license-policy-file "$ROOT_POLICY_MISMATCH_LICENSE" \
  >"$TMP_DIR/policy-mismatch.out" \
  2>"$TMP_DIR/policy-mismatch.err"
then
  echo "expected workspace license policy checker to fail on root policy marker drift" >&2
  cat "$TMP_DIR/policy-mismatch.out" >&2 || true
  cat "$TMP_DIR/policy-mismatch.err" >&2 || true
  exit 1
fi

if ! grep -q "license_policy_marker_mismatch" "$TMP_DIR/policy-mismatch.err"; then
  echo "expected license_policy_marker_mismatch reason in checker stderr output" >&2
  cat "$TMP_DIR/policy-mismatch.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=license_policy_marker_mismatch$' "$TMP_DIR/policy-mismatch.out"; then
  echo "expected deterministic license_policy_marker_mismatch reason code marker in checker stdout output" >&2
  cat "$TMP_DIR/policy-mismatch.out" >&2 || true
  exit 1
fi

MISMATCH_MANIFEST="$TMP_DIR/mismatch.Cargo.toml"
cp "$ROOT_DIR/crates/kamn-core/Cargo.toml" "$MISMATCH_MANIFEST"
python3 - "$MISMATCH_MANIFEST" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(r'^license\s*=\s*".*"$', 'license = "MIT"', text, flags=re.MULTILINE)
path.write_text(text, encoding="utf-8")
PY

if python3 "$CHECKER" --manifest "$MISMATCH_MANIFEST" --expected-license "Apache-2.0" --license-policy-file "$ROOT_DIR/LICENSE" >"$TMP_DIR/mismatch.out" 2>"$TMP_DIR/mismatch.err"; then
  echo "expected workspace license policy checker to fail on mismatched license field" >&2
  cat "$TMP_DIR/mismatch.out" >&2 || true
  cat "$TMP_DIR/mismatch.err" >&2 || true
  exit 1
fi

if ! grep -q "license_mismatch" "$TMP_DIR/mismatch.err"; then
  echo "expected license_mismatch reason in checker stderr output" >&2
  cat "$TMP_DIR/mismatch.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=license_mismatch$' "$TMP_DIR/mismatch.out"; then
  echo "expected deterministic license_mismatch reason code marker in checker stdout output" >&2
  cat "$TMP_DIR/mismatch.out" >&2 || true
  exit 1
fi
if ! grep -q '^reason_class=metadata_mismatch$' "$TMP_DIR/mismatch.out"; then
  echo "expected deterministic metadata_mismatch reason-class marker in checker stdout output" >&2
  cat "$TMP_DIR/mismatch.out" >&2 || true
  exit 1
fi

MISSING_MANIFEST="$TMP_DIR/missing.Cargo.toml"
cp "$ROOT_DIR/crates/kamn-core/Cargo.toml" "$MISSING_MANIFEST"
python3 - "$MISSING_MANIFEST" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(r'^license\s*=\s*".*"$\n?', '', text, flags=re.MULTILINE)
path.write_text(text, encoding="utf-8")
PY

if python3 "$CHECKER" --manifest "$MISSING_MANIFEST" --expected-license "Apache-2.0" --license-policy-file "$ROOT_DIR/LICENSE" >"$TMP_DIR/missing.out" 2>"$TMP_DIR/missing.err"; then
  echo "expected workspace license policy checker to fail on missing license field" >&2
  cat "$TMP_DIR/missing.out" >&2 || true
  cat "$TMP_DIR/missing.err" >&2 || true
  exit 1
fi

if ! grep -q "license_missing" "$TMP_DIR/missing.err"; then
  echo "expected license_missing reason in checker stderr output" >&2
  cat "$TMP_DIR/missing.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=license_missing$' "$TMP_DIR/missing.out"; then
  echo "expected deterministic license_missing reason code marker in checker stdout output" >&2
  cat "$TMP_DIR/missing.out" >&2 || true
  exit 1
fi

MALFORMED_MANIFEST="$TMP_DIR/malformed.Cargo.toml"
cat >"$MALFORMED_MANIFEST" <<'EOF'
[package
name = "broken"
EOF

if python3 "$CHECKER" --manifest "$MALFORMED_MANIFEST" --expected-license "Apache-2.0" --license-policy-file "$ROOT_DIR/LICENSE" >"$TMP_DIR/malformed.out" 2>"$TMP_DIR/malformed.err"; then
  echo "expected workspace license policy checker to fail on malformed Cargo manifest" >&2
  cat "$TMP_DIR/malformed.out" >&2 || true
  cat "$TMP_DIR/malformed.err" >&2 || true
  exit 1
fi

if ! grep -q "manifest_invalid_toml" "$TMP_DIR/malformed.err"; then
  echo "expected manifest_invalid_toml reason in checker stderr output" >&2
  cat "$TMP_DIR/malformed.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=manifest_invalid_toml$' "$TMP_DIR/malformed.out"; then
  echo "expected deterministic manifest_invalid_toml reason code marker in checker stdout output" >&2
  cat "$TMP_DIR/malformed.out" >&2 || true
  exit 1
fi

PACKAGE_MISSING_MANIFEST="$TMP_DIR/package-missing.Cargo.toml"
cp "$ROOT_DIR/crates/kamn-core/Cargo.toml" "$PACKAGE_MISSING_MANIFEST"
python3 - "$PACKAGE_MISSING_MANIFEST" <<'PY'
import pathlib
import re
import sys

path = pathlib.Path(sys.argv[1])
text = path.read_text(encoding="utf-8")
text = re.sub(r'^\[package\]\n', '', text, count=1, flags=re.MULTILINE)
path.write_text(text, encoding="utf-8")
PY

if python3 "$CHECKER" --manifest "$PACKAGE_MISSING_MANIFEST" --expected-license "Apache-2.0" --license-policy-file "$ROOT_DIR/LICENSE" >"$TMP_DIR/package-missing.out" 2>"$TMP_DIR/package-missing.err"; then
  echo "expected workspace license policy checker to fail when package section is missing" >&2
  cat "$TMP_DIR/package-missing.out" >&2 || true
  cat "$TMP_DIR/package-missing.err" >&2 || true
  exit 1
fi

if ! grep -q "package_section_missing" "$TMP_DIR/package-missing.err"; then
  echo "expected package_section_missing reason in checker stderr output" >&2
  cat "$TMP_DIR/package-missing.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=package_section_missing$' "$TMP_DIR/package-missing.out"; then
  echo "expected deterministic package_section_missing reason code marker in checker stdout output" >&2
  cat "$TMP_DIR/package-missing.out" >&2 || true
  exit 1
fi

MISSING_PATH_MANIFEST="$TMP_DIR/does-not-exist.Cargo.toml"
if python3 "$CHECKER" --manifest "$MISSING_PATH_MANIFEST" --expected-license "Apache-2.0" --license-policy-file "$ROOT_DIR/LICENSE" >"$TMP_DIR/manifest-not-found.out" 2>"$TMP_DIR/manifest-not-found.err"; then
  echo "expected workspace license policy checker to fail on missing manifest path" >&2
  cat "$TMP_DIR/manifest-not-found.out" >&2 || true
  cat "$TMP_DIR/manifest-not-found.err" >&2 || true
  exit 1
fi

if ! grep -q "manifest_not_found" "$TMP_DIR/manifest-not-found.err"; then
  echo "expected manifest_not_found reason in checker stderr output" >&2
  cat "$TMP_DIR/manifest-not-found.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=manifest_not_found$' "$TMP_DIR/manifest-not-found.out"; then
  echo "expected deterministic manifest_not_found reason code marker in checker stdout output" >&2
  cat "$TMP_DIR/manifest-not-found.out" >&2 || true
  exit 1
fi

MISSING_POLICY_FILE="$TMP_DIR/does-not-exist.LICENSE"
if python3 "$CHECKER" --workspace-root "$ROOT_DIR" --expected-license "Apache-2.0" --license-policy-file "$MISSING_POLICY_FILE" >"$TMP_DIR/policy-not-found.out" 2>"$TMP_DIR/policy-not-found.err"; then
  echo "expected workspace license policy checker to fail when root policy file path is missing" >&2
  cat "$TMP_DIR/policy-not-found.out" >&2 || true
  cat "$TMP_DIR/policy-not-found.err" >&2 || true
  exit 1
fi

if ! grep -q "license_policy_file_not_found" "$TMP_DIR/policy-not-found.err"; then
  echo "expected license_policy_file_not_found reason in checker stderr output" >&2
  cat "$TMP_DIR/policy-not-found.err" >&2 || true
  exit 1
fi
if ! grep -q '^reason_codes_csv=license_policy_file_not_found$' "$TMP_DIR/policy-not-found.out"; then
  echo "expected deterministic license_policy_file_not_found reason code marker in checker stdout output" >&2
  cat "$TMP_DIR/policy-not-found.out" >&2 || true
  exit 1
fi

if python3 "$CHECKER" --workspace-root "$ROOT_DIR" --expected-license "Apache-2.0" --license-policy-file "$ROOT_DIR/LICENSE" --lane-profile local-heavy >"$TMP_DIR/local-heavy-no-opt-in.out" 2>"$TMP_DIR/local-heavy-no-opt-in.err"; then
  echo "expected workspace license policy checker to fail when local-heavy mode is not explicitly opted in" >&2
  cat "$TMP_DIR/local-heavy-no-opt-in.out" >&2 || true
  cat "$TMP_DIR/local-heavy-no-opt-in.err" >&2 || true
  exit 1
fi

if ! grep -q '^reason_codes_csv=metadata_governance_local_heavy_opt_in_required$' "$TMP_DIR/local-heavy-no-opt-in.out"; then
  echo "expected deterministic local-heavy opt-in reason code marker in checker output" >&2
  cat "$TMP_DIR/local-heavy-no-opt-in.out" >&2 || true
  exit 1
fi
if ! grep -q '^reason_class=boundary$' "$TMP_DIR/local-heavy-no-opt-in.out"; then
  echo "expected deterministic boundary reason-class marker in checker output" >&2
  cat "$TMP_DIR/local-heavy-no-opt-in.out" >&2 || true
  exit 1
fi
if ! grep -q '^ci_smoke_local_heavy_boundary_status=violation$' "$TMP_DIR/local-heavy-no-opt-in.out"; then
  echo "expected ci/local boundary violation marker in checker output" >&2
  cat "$TMP_DIR/local-heavy-no-opt-in.out" >&2 || true
  exit 1
fi
if ! grep -q '^local_heavy_lane_execution_mode=blocked$' "$TMP_DIR/local-heavy-no-opt-in.out"; then
  echo "expected blocked local-heavy execution-mode marker in checker output" >&2
  cat "$TMP_DIR/local-heavy-no-opt-in.out" >&2 || true
  exit 1
fi

local_heavy_opt_in_output="$(
  python3 "$CHECKER" \
    --workspace-root "$ROOT_DIR" \
    --expected-license "Apache-2.0" \
    --license-policy-file "$ROOT_DIR/LICENSE" \
    --lane-profile local-heavy \
    --local-heavy-opt-in
)"
if ! printf '%s\n' "$local_heavy_opt_in_output" | grep -q '^ci_smoke_local_heavy_boundary_status=verified$'; then
  echo "expected ci/local boundary verified marker for local-heavy opt-in run" >&2
  exit 1
fi
if ! printf '%s\n' "$local_heavy_opt_in_output" | grep -q '^ci_smoke_lane_cost_profile=not-applicable$'; then
  echo "expected ci smoke cost-profile marker to be not-applicable in local-heavy opt-in run" >&2
  exit 1
fi
if ! printf '%s\n' "$local_heavy_opt_in_output" | grep -q '^local_heavy_lane_execution_mode=opt_in$'; then
  echo "expected local-heavy execution-mode opt_in marker in local-heavy run output" >&2
  exit 1
fi

echo "workspace license policy checker tests passed."
