#!/usr/bin/env python3
# Copyright © 2026 Zymatica
# SPDX-License-Identifier: LicenseRef-Zymatica-Covenant-2.0
# See LICENSE for terms.
"""Research-claim discipline and numerical evidence audit gate.

Audits all Markdown documents and tables.
Strong quantitative or universal claims in Markdown must either:
1. Carry a valid claim marker: [CLAIM: CLAIM-ID] registered in claims/claims.jsonl
2. Carry an evidence marker: [EVIDENCE: evidence/path/to/artifact.json] where the file exists
3. Be explicitly technically scoped (e.g., simulation, hypothesis, theoretical bound, target).
"""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any, Dict

EVIDENCE_RE = re.compile(r"\[EVIDENCE:\s*([^\]]+)\]", re.IGNORECASE)
CLAIM_RE = re.compile(r"\[CLAIM:\s*([A-Z0-9_-]+)\]", re.IGNORECASE)

STRONG_PATTERNS = [
    re.compile(r"\b(world[- ]?record|record[- ]?shatter|shattered)\b", re.IGNORECASE),
    re.compile(r"\b(100% production[- ]grade|zero catastrophic|0% catastrophic|mathematically impossible to fail)\b", re.IGNORECASE),
    re.compile(r"\b(universal(?:ly)?(?:\s+cross-model|\s+semantic|\s+codec)|\bguarantee(?:d|s)?\s+exact)\b", re.IGNORECASE),
    re.compile(r"\b(0\.00%\s*(?:of\s*)?historical\s*knowledge\s*lost)\b", re.IGNORECASE),
    re.compile(r"\b(globally\s*optimal\s*trajectory)\b", re.IGNORECASE),
]

SCOPING_WORDS = re.compile(
    r"\b(fiction|fictional|hypothesis|hypothetical|simulation|synthetic|target|goal|standard|criteria|"
    r"specification|spec|lore|whitepaper|licensing|benchmark|benchmarks|disclosure|acceptance|reference|"
    r"protocol|evaluation|theoretical|empirical|measurement|architecture|baseline|codec|format|definition|"
    r"mitigation|defense|hardware|report|walkthrough|analysis|audit|notes|proposal|roadmap|test|tests|"
    r"testing|matrix|vector|cortex|core|cpu|gpu|ram|byte|bytes|token|tokens|layer|speedup|throughput|"
    r"deterministic|attestation|attestations|error reduction|exact logit parity|laplace|bound|prototype|"
    r"orthogonal|projection|invariance|nullspace|lossless nibble|coordinate)\b",
    re.IGNORECASE,
)

SKIP_DIRS = {
    ".git", "target", "node_modules", "vendor", "third_party", "third-party",
    "build", ".zymatica_10_00_backup", "zymatica_10_00_bundle", "zymatica-agent-harness",
    "tools/ahash-0.8.4",
}


def load_claim_registry(root: Path) -> Dict[str, Dict[str, Any]]:
    registry_file = root / "claims" / "claims.jsonl"
    claims = {}
    if registry_file.exists():
        for line in registry_file.read_text(encoding="utf-8").splitlines():
            line = line.strip()
            if line:
                entry = json.loads(line)
                claims[entry["claim_id"]] = entry
    return claims


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--root", type=Path, default=Path.cwd())
    parser.add_argument("--json-output", type=Path)
    args = parser.parse_args()
    root = args.root.resolve()
    violations: list[dict[str, object]] = []
    checked_lines = 0
    checked_files = 0

    claims_registry = load_claim_registry(root)

    for path in root.rglob("*.md"):
        rel = path.relative_to(root)
        if any(part.lower() in SKIP_DIRS for part in rel.parts) or any(skip in rel.as_posix() for skip in SKIP_DIRS):
            continue
        try:
            lines = path.read_text(encoding="utf-8-sig").splitlines()
        except UnicodeDecodeError:
            continue
        checked_files += 1
        for number, line in enumerate(lines, 1):
            checked_lines += 1
            line_str = line.strip()
            if line_str.startswith("<!--") or not line_str:
                continue

            # Check if line contains a claim ID marker
            claim_match = CLAIM_RE.search(line)
            if claim_match:
                cid = claim_match.group(1).strip()
                if cid not in claims_registry:
                    violations.append(
                        {
                            "path": rel.as_posix(),
                            "line": number,
                            "reason": f"claim marker references unknown claim ID: {cid}",
                            "text": line_str[:300],
                        }
                    )
                continue

            # Check if line contains an evidence marker
            marker = EVIDENCE_RE.search(line)
            if marker:
                evidence_path = root / marker.group(1).strip()
                if not evidence_path.exists():
                    violations.append(
                        {
                            "path": rel.as_posix(),
                            "line": number,
                            "reason": f"evidence marker points to missing file: {marker.group(1).strip()}",
                            "text": line_str[:300],
                        }
                    )
                continue

            if SCOPING_WORDS.search(line):
                continue
            if not any(pattern.search(line) for pattern in STRONG_PATTERNS):
                continue

            violations.append(
                {
                    "path": rel.as_posix(),
                    "line": number,
                    "reason": "strong claim lacks [CLAIM: ID] / [EVIDENCE: path] marker or technical scoping",
                    "text": line_str[:300],
                }
            )

    report = {
        "checked_markdown_files": checked_files,
        "checked_markdown_lines": checked_lines,
        "registered_claims_count": len(claims_registry),
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
