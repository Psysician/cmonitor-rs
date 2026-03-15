#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "tests/vendor/claude-code-usage-monitor.manifest.json"
SKIP_DIRS = {"__pycache__"}


def load_manifest() -> dict:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


def bundle_path(manifest: dict) -> Path:
    return REPO_ROOT / manifest["bundle_dir"]


def oracle_python() -> Path:
    candidate = REPO_ROOT.parent / "c-monitor" / ".venv" / "bin" / "python"
    if candidate.exists():
        return candidate
    return Path(sys.executable)


def manifest_environment(manifest: dict, bundle: Path) -> dict[str, str]:
    """Keeps bundle execution aligned with manifest-pinned defaults and import
    roots so parity runs exercise the vendored oracle exactly. (ref: DL-001)"""
    env = os.environ.copy()
    env.update({key: str(value) for key, value in manifest.get("oracle_env", {}).items()})
    env["PYTHONPATH"] = str(bundle / "src")
    return env


def expand_inventory(base: Path, manifest: dict) -> list[str]:
    """Builds the manifest inventory that defines the executable oracle surface,
    including import roots beyond the entrypoint file. (ref: DL-001)"""
    inventory: list[str] = []

    for relative in manifest.get("required_files", []):
        candidate = base / relative
        if not candidate.exists():
            raise SystemExit(f"vendored oracle missing manifest file: {relative}")
        inventory.append(relative)

    for relative in manifest.get("required_roots", []):
        root = base / relative
        if not root.exists():
            raise SystemExit(f"vendored oracle missing manifest root: {relative}")
        if root.is_file():
            inventory.append(relative)
            continue
        for child in sorted(root.rglob("*.py")):
            if any(part in SKIP_DIRS for part in child.parts):
                continue
            inventory.append(child.relative_to(base).as_posix())

    return sorted(dict.fromkeys(inventory))


def validate_bundle(manifest: dict, bundle: Path) -> None:
    expand_inventory(bundle, manifest)

    pin_file = bundle / ".oracle-pin"
    if not pin_file.exists():
        raise SystemExit("vendored oracle pin file is missing")
    if pin_file.read_text(encoding="utf-8").strip() != manifest["source"]["commit"]:
        raise SystemExit("vendored oracle pin does not match manifest")

    completed = subprocess.run(
        [str(oracle_python()), "-c", "import claude_monitor.cli.main"],
        env=manifest_environment(manifest, bundle),
        text=True,
        capture_output=True,
        check=False,
    )
    if completed.returncode != 0:
        raise SystemExit(
            "vendored oracle import check failed:\n" + completed.stderr.strip()
        )


def materialize_checkout(source: Path, destination: Path, manifest: dict) -> None:
    """Copies only the manifest inventory so the vendored bundle matches the
    upstream modules required by bundle-mode parity. (ref: DL-001)"""
    inventory = expand_inventory(source, manifest)
    if destination.exists():
        shutil.rmtree(destination)
    destination.mkdir(parents=True, exist_ok=True)
    for relative in inventory:
        src = source / relative
        dest = destination / relative
        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(src, dest)
    (destination / ".oracle-pin").write_text(
        manifest["source"]["commit"] + "\n",
        encoding="utf-8",
    )


def fetch_from_git(manifest: dict, destination: Path) -> None:
    with tempfile.TemporaryDirectory(prefix="cmonitor-oracle-") as temp_dir:
        checkout = Path(temp_dir) / "upstream"
        subprocess.run(
            ["git", "clone", manifest["source"]["repo"], str(checkout)],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(checkout), "checkout", manifest["source"]["commit"]],
            check=True,
        )
        materialize_checkout(checkout, destination, manifest)


def ensure_bundle(check_pin: bool) -> Path:
    manifest = load_manifest()
    destination = bundle_path(manifest)
    if destination.exists():
        try:
            validate_bundle(manifest, destination)
            return destination
        except SystemExit:
            if check_pin:
                raise
            shutil.rmtree(destination)

    if check_pin:
        raise SystemExit("vendored oracle bundle is missing")

    override_env = manifest["source"].get("local_override_env")
    if override_env and os.environ.get(override_env):
        materialize_checkout(
            Path(os.environ[override_env]),
            destination,
            manifest,
        )
    else:
        fetch_from_git(manifest, destination)

    validate_bundle(manifest, destination)
    return destination


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--check-pin", action="store_true")
    args = parser.parse_args()
    bundle = ensure_bundle(check_pin=args.check_pin)
    print(json.dumps({"bundle": str(bundle.relative_to(REPO_ROOT))}, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
