#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
MANIFEST_PATH = REPO_ROOT / "tests/vendor/claude-code-usage-monitor.manifest.json"
FIXTURE_ROOT = REPO_ROOT / "tests/fixtures/contract"

SCENARIOS = {
    "cli-defaults": {"fixture": "defaults", "args": []},
    "cli-overrides": {
        "fixture": "overrides",
        "args": [
            "--plan",
            "max20",
            "--view",
            "daily",
            "--theme",
            "dark",
            "--timezone",
            "UTC",
            "--refresh-rate",
            "5",
            "--refresh-per-second",
            "1.5",
        ],
    },
    "cli-version": {"fixture": "defaults", "args": ["--version"]},
    "cli-clear": {"fixture": "clear", "args": ["--clear"]},
}


def manifest() -> dict:
    return json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))


def bundle_path() -> Path:
    return REPO_ROOT / manifest()["bundle_dir"]


def oracle_python() -> Path:
    candidate = REPO_ROOT.parent / "c-monitor" / ".venv" / "bin" / "python"
    if candidate.exists():
        return candidate
    return Path(sys.executable)


def scenario_fixture_home(name: str, override: str | None) -> Path:
    if override:
        return Path(override)
    fixture = SCENARIOS[name]["fixture"]
    return FIXTURE_ROOT / fixture / "home"


def load_last_used(home: Path) -> dict | None:
    last_used_path = home / ".claude-monitor" / "last_used.json"
    if not last_used_path.exists():
        return None
    payload = json.loads(last_used_path.read_text(encoding="utf-8"))
    payload.pop("timestamp", None)
    return payload


def ensure_bundle() -> None:
    """Fails fast when the vendored oracle cannot import, so parity never
    degrades into a synthetic fallback path. (ref: DL-001)"""
    completed = subprocess.run(
        [
            sys.executable,
            str(REPO_ROOT / "tests/support/fetch_upstream_oracle.py"),
            "--check-pin",
        ],
        cwd=REPO_ROOT,
        text=True,
        capture_output=True,
        check=True,
    )
    if completed.stdout:
        # Keep the runner stdout reserved for its JSON payload. (ref: DL-001)
        _ = completed.stdout


def run_bundle(name: str, argv: list[str], fixture_home: Path) -> dict:
    """Runs the pinned upstream CLI under the fixture home so payloads reflect
    executable oracle behavior across settings flows. (ref: DL-001)"""
    ensure_bundle()
    fixture_home.mkdir(parents=True, exist_ok=True)
    command = [
        str(oracle_python()),
        "-c",
        "from claude_monitor.cli.main import main; import sys; raise SystemExit(main(sys.argv[1:]))",
        *argv,
    ]
    env = os.environ.copy()
    env.update({key: str(value) for key, value in manifest().get("oracle_env", {}).items()})
    env["HOME"] = str(fixture_home)
    env["PYTHONPATH"] = str(bundle_path() / "src")
    completed = subprocess.run(command, env=env, text=True, capture_output=True, check=False)
    if completed.returncode != 0 and (
        "ModuleNotFoundError" in completed.stderr or "ImportError" in completed.stderr
    ):
        raise SystemExit(
            f"vendored oracle import failed for {name}:\n{completed.stderr.strip()}"
        )

    return {
        "scenario": name,
        "args": argv,
        "exit_code": completed.returncode,
        "stdout": completed.stdout,
        "stderr": completed.stderr,
        "last_used": load_last_used(fixture_home),
        "source_commit": manifest()["source"]["commit"],
        "oracle_mode": "bundle",
    }


def run_scenario(name: str, fixture_home_override: str | None) -> dict:
    scenario = SCENARIOS[name]
    fixture_home = scenario_fixture_home(name, fixture_home_override)
    return run_bundle(name, scenario["args"], fixture_home)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--scenario", choices=sorted(SCENARIOS), required=True)
    parser.add_argument("--fixture-home")
    args = parser.parse_args()
    print(json.dumps(run_scenario(args.scenario, args.fixture_home), indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
