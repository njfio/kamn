#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
DISPATCHER="$ROOT_DIR/scripts/lib/exec_dispatch.sh"
REGISTRY="$ROOT_DIR/scripts/lib/exec_registry.json"

if [[ ! -x "$DISPATCHER" ]]; then
  echo "expected exec dispatcher to be executable: $DISPATCHER" >&2
  exit 1
fi

if [[ ! -f "$REGISTRY" ]]; then
  echo "expected exec registry file: $REGISTRY" >&2
  exit 1
fi

python3 - "$ROOT_DIR" "$DISPATCHER" "$REGISTRY" <<'PY'
import json
import os
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path

root = Path(sys.argv[1])
dispatcher = Path(sys.argv[2]).resolve()
registry_path = Path(sys.argv[3])

payload = json.loads(registry_path.read_text(encoding="utf-8"))
entries = payload.get("entries")
if not isinstance(entries, dict) or not entries:
    raise SystemExit("expected non-empty entries map in exec registry")

invalid_entries: list[str] = []
declarative_policy_migration_v1_candidates: list[str] = []
for wrapper_rel, entry in sorted(entries.items()):
    wrapper_path = root / wrapper_rel
    if not wrapper_path.exists():
        invalid_entries.append(f"missing wrapper path: {wrapper_rel}")
        continue
    if not wrapper_path.is_symlink():
        invalid_entries.append(f"wrapper is not symlink: {wrapper_rel}")
        continue
    if wrapper_path.resolve() != dispatcher:
        invalid_entries.append(
            f"wrapper does not resolve to dispatcher: {wrapper_rel} -> {wrapper_path.resolve()}"
        )

    if not isinstance(entry, dict):
        invalid_entries.append(f"registry entry is not object: {wrapper_rel}")
        continue
    interpreter = entry.get("interpreter")
    target = entry.get("target")
    args_prefix = entry.get("args_prefix")
    passthrough = entry.get("passthrough")
    if interpreter not in {"python3", "bash"}:
        invalid_entries.append(f"invalid interpreter for {wrapper_rel}: {interpreter!r}")
    if not isinstance(target, str) or target.strip() == "":
        invalid_entries.append(f"invalid target for {wrapper_rel}: {target!r}")
    else:
        target_path = root / target
        if not target_path.exists():
            invalid_entries.append(f"target path missing for {wrapper_rel}: {target}")
    if not isinstance(args_prefix, list) or any(not isinstance(item, str) for item in args_prefix):
        invalid_entries.append(f"invalid args_prefix for {wrapper_rel}")
    if not isinstance(passthrough, bool):
        invalid_entries.append(f"invalid passthrough for {wrapper_rel}: {passthrough!r}")

    if (
        isinstance(wrapper_rel, str)
        and wrapper_rel.startswith("scripts/")
        and ("/check_" in wrapper_rel or "/validate_" in wrapper_rel)
        and wrapper_rel.endswith(".sh")
        and interpreter == "python3"
        and isinstance(target, str)
        and (target.endswith("_contract.py") or target.endswith("_policy_contract.py"))
    ):
        target_path = root / target
        if target_path.is_file():
            line_count = sum(1 for _ in target_path.read_text(encoding="utf-8").splitlines())
            if line_count <= 1500:
                declarative_policy_migration_v1_candidates.append(wrapper_rel)

if invalid_entries:
    raise SystemExit("\n".join(invalid_entries))

if len(declarative_policy_migration_v1_candidates) != 112:
    preview = "\n".join(declarative_policy_migration_v1_candidates[:80])
    raise SystemExit(
        "expected 112 declarative policy migration v1 wrapper candidates, "
        f"found {len(declarative_policy_migration_v1_candidates)}\n{preview}"
    )

dispatcher_py = root / "scripts/lib/exec_dispatch.py"
checker_py = root / "scripts/framework/declarative_policy_checker.py"
framework_contract_py = root / "scripts/framework/contract_framework.py"
framework_init_py = root / "scripts/framework/__init__.py"

with tempfile.TemporaryDirectory() as temp_dir:
    temp_root = Path(temp_dir)
    (temp_root / "scripts/lib").mkdir(parents=True, exist_ok=True)
    (temp_root / "scripts/framework").mkdir(parents=True, exist_ok=True)
    (temp_root / "scripts/tmp").mkdir(parents=True, exist_ok=True)

    # Copy only files required for delegated checker execution in this sandbox.
    (temp_root / "scripts/framework/declarative_policy_checker.py").write_text(
        checker_py.read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    (temp_root / "scripts/framework/contract_framework.py").write_text(
        framework_contract_py.read_text(encoding="utf-8"),
        encoding="utf-8",
    )
    (temp_root / "scripts/framework/__init__.py").write_text(
        framework_init_py.read_text(encoding="utf-8"),
        encoding="utf-8",
    )

    wrapper_path = temp_root / "scripts/tmp/validate_demo_policy.sh"
    wrapper_path.write_text("#!/usr/bin/env bash\nexit 0\n", encoding="utf-8")
    os.chmod(wrapper_path, 0o755)

    target_path = temp_root / "scripts/tmp/demo_contract.py"
    target_path.write_text(
        "\n".join(
            (
                "#!/usr/bin/env python3",
                "import os",
                "import sys",
                "print(f\"delegate_env={os.getenv('KAMN_DECLARATIVE_POLICY_CHECKER_DELEGATE', '0')}\")",
                "print(\"argv=\" + \" \".join(sys.argv[1:]))",
            )
        )
        + "\n",
        encoding="utf-8",
    )
    os.chmod(target_path, 0o755)

    sandbox_registry = temp_root / "scripts/lib/exec_registry.json"
    sandbox_registry.write_text(
        json.dumps(
            {
                "version": 1,
                "entries": {
                    "scripts/tmp/validate_demo_policy.sh": {
                        "interpreter": "python3",
                        "target": "scripts/tmp/demo_contract.py",
                        "args_prefix": ["check", "--repo-root", "${KAMN_ROOT}"],
                        "passthrough": True,
                    }
                },
            },
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )

    command = [
        "python3",
        str(dispatcher_py),
        "--registry",
        str(sandbox_registry),
        "--invoked-path",
        str(wrapper_path),
        "--",
        "--report-file",
        "/tmp/demo-report.json",
    ]
    result = subprocess.run(command, check=False, capture_output=True, text=True)
    if result.returncode != 0:
        raise SystemExit(
            "expected declarative compatibility delegation command to succeed:\n"
            f"stdout:\n{result.stdout}\n"
            f"stderr:\n{result.stderr}"
        )

    if "delegate_env=1" not in result.stdout:
        raise SystemExit(
            "expected delegated checker execution marker delegate_env=1 in dispatcher output"
        )
    expected_prefix = f"argv=check --repo-root {temp_root} --report-file /tmp/demo-report.json"
    if expected_prefix not in result.stdout:
        raise SystemExit(
            "expected delegated checker to preserve args_prefix token expansion + passthrough forwarding"
        )

prefixes = ("$ROOT_DIR/", "$KAMN_ROOT/", "${ROOT_DIR}/", "${KAMN_ROOT}/")
remaining_tiny_wrappers: list[str] = []
for script in sorted((root / "scripts").rglob("*.sh")):
    if script.is_symlink():
        continue
    lines = script.read_text(encoding="utf-8").splitlines()
    exec_lines = [line.strip() for line in lines if line.strip().startswith("exec ")]
    if len(exec_lines) != 1:
        continue
    line = exec_lines[0]
    if line.endswith("\\"):
        continue
    tokens = shlex.split(line)
    if len(tokens) < 3 or tokens[0] != "exec" or tokens[1] not in {"python3", "bash"}:
        continue
    target = tokens[2]
    if not target.startswith(prefixes):
        continue
    rel = script.relative_to(root).as_posix()
    if rel == "scripts/kolme/contract_lane_dispatch_impl.sh":
        # This file is an internal kolm dispatch bridge, not a registry-backed wrapper.
        continue
    remaining_tiny_wrappers.append(rel)

if remaining_tiny_wrappers:
    message = ["expected zero remaining tiny exec wrappers after registry migration"]
    message.extend(remaining_tiny_wrappers[:80])
    if len(remaining_tiny_wrappers) > 80:
        message.append(f"... and {len(remaining_tiny_wrappers) - 80} more")
    raise SystemExit("\n".join(message))
PY

echo "exec wrapper registry migration tests passed."
