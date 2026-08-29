#!/usr/bin/env python3
# Copyright © 2026 Zymatica
# SPDX-License-Identifier: LicenseRef-Zymatica-Covenant-2.0
# See LICENSE for terms.
"""Research-claim discipline gate.

Strong quantitative or universal claims in Markdown must either be explicitly scoped as fiction /
hypothesis / simulation, or carry an evidence marker of the form:
    [EVIDENCE: evidence/path/to/artifact.json]
The referenced path must exist.
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

EVIDENCE_RE = re.compile(r"\[EVIDENCE:\s*([^\]]+)\]", re.IGNORECASE)
STRONG_PATTERNS = [
    re.compile(r"\b(world[- ]?record|record[- ]?shatter|shattered)\b", re.IGNORECASE),
    re.compile(r"\b(100%|zero failures?|0 failures?|0% catastrophic|zero catastrophic)\b", re.IGNORECASE),
    re.compile(r"\b(universal(?:ly)?|guarantee(?:d|s)?|proves? exact|mathematically impossible to fail)\b", re.IGNORECASE),
    re.compile(r"\b\d+(?:\.\d+)?x\b", re.IGNORECASE),
    re.compile(r"\b\d{1,3}(?:,\d{3}){2,}\b"),
]
SCOPING_WORDS = re.compile(
    r"\b(fiction|fictional|hypothesis|hypothetical|simulation|synthetic|target|goal|standard|criteria|specification|spec|lore|whitepaper|licensing|benchmark|disclosure|acceptance)\b",
    re.IGNORECASE,
)
SKIP_DIRS = {
    ".git", "target", "node_modules", "vendor", "third_party", "third-party",
    "build", ".zymatica_10_00_backup", "zymatica_10_00_bundle", "zymatica-agent-harness",
    "zymatica-language-u", "evidence", "crates", "tools", "patches", "docs",
}


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--json-output", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    violations: list[dict[str, object]] = []
    checked_lines = 0

    for path in root.rglob("*.md"):
        rel = path.relative_to(root)
        if any(part.lower() in SKIP_DIRS for part in rel.parts):
            continue
        try:
            lines = path.read_text(encoding="utf-8-sig").splitlines()
        except UnicodeDecodeError:
            continue
        for number, line in enumerate(lines, 1):
            checked_lines += 1
            if SCOPING_WORDS.search(line):
                continue
            if not any(pattern.search(line) for pattern in STRONG_PATTERNS):
                continue
            marker = EVIDENCE_RE.search(line)
            if marker is None:
                violations.append(
                    {
                        "path": rel.as_posix(),
                        "line": number,
                        "reason": "strong claim lacks [EVIDENCE: path] marker",
                        "text": line.strip()[:300],
                    }
                )
                continue
            evidence_path = root / marker.group(1).strip()
            if not evidence_path.exists():
                violations.append(
                    {
                        "path": rel.as_posix(),
                        "line": number,
                        "reason": f"evidence marker points to missing file: {marker.group(1).strip()}",
                        "text": line.strip()[:300],
                    }
                )

    report = {
        "checked_markdown_lines": checked_lines,
        "violations": violations,
        "status": "PASS" if not violations else "FAIL",
    }
    if args.json_output:
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if not violations else 1


if __name__ == "__main__":
    raise SystemExit(main())
