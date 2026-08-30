#!/usr/bin/env python3
# Copyright © 2026 Zymatica
# SPDX-License-Identifier: LicenseRef-Zymatica-Covenant-2.0
# See LICENSE for terms.
"""Create a cryptographic manifest for a Zymatica evidence directory with source tree binding."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shutil
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


def sha256_file(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as f:
        for chunk in iter(lambda: f.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def compute_source_tree_hash(root: Path) -> str:
    """Compute deterministic SHA-256 tree hash over versioned source files."""
    h = hashlib.sha256()
    tracked_exts = {".rs", ".py", ".toml", ".json", ".lean", ".cpp", ".hpp", ".h", ".c"}
    skip_dirs = {".git", "target", "node_modules", "build", ".zymatica_10_00_backup", "zymatica_10_00_bundle"}
    for p in sorted(root.rglob("*")):
        if p.is_file() and p.suffix in tracked_exts:
            if not any(part in skip_dirs for part in p.parts):
                rel = p.relative_to(root).as_posix()
                h.update(rel.encode("utf-8"))
                h.update(sha256_file(p).encode("utf-8"))
    return h.hexdigest()


def command_output(command: list[str], cwd: Path) -> str | None:
    env = os.environ.copy()
    cargo_bin = os.path.expanduser("~/.cargo/bin")
    if os.path.isdir(cargo_bin) and cargo_bin not in env.get("PATH", ""):
        env["PATH"] = f"{cargo_bin}{os.pathsep}{env.get('PATH', '')}"
    cmd = list(command)
    exe = shutil.which(cmd[0], path=env.get("PATH"))
    if exe:
        cmd[0] = exe
    try:
        result = subprocess.run(cmd, cwd=cwd, env=env, text=True, capture_output=True, check=True)
        return result.stdout.strip()
    except Exception:
        return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("evidence_dir", type=Path)
    parser.add_argument("--repo", type=Path, default=Path.cwd())
    parser.add_argument("--output", type=Path)
    args = parser.parse_args()
    repo = args.repo.resolve()
    evidence_dir = args.evidence_dir.resolve()
    if not evidence_dir.is_dir():
        raise SystemExit(f"not a directory: {evidence_dir}")

    files = []
    for path in sorted(p for p in evidence_dir.rglob("*") if p.is_file()):
        if args.output and path.resolve() == args.output.resolve():
            continue
        if path.name in {"MANIFEST.json", "SHA256SUMS", "release_attestation.json"}:
            continue
        files.append(
            {
                "path": path.relative_to(evidence_dir).as_posix(),
                "size": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )

    source_tree_sha = compute_source_tree_hash(repo)

    payload = {
        "schema": "zymatica.evidence-manifest.v2",
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "source_tree_sha256": source_tree_sha,
        "git_head": command_output(["git", "rev-parse", "HEAD"], repo),
        "git_status_porcelain": command_output(["git", "status", "--porcelain"], repo),
        "rustc": command_output(["rustc", "--version"], repo),
        "cargo": command_output(["cargo", "--version"], repo),
        "python": sys.version,
        "platform": platform.platform(),
        "files": files,
    }
    output = args.output or (evidence_dir / "MANIFEST.json")
    output.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
