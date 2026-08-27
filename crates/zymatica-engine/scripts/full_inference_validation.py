#!/usr/bin/env python3
"""Run full native inference validation against local Gemma E2B/E4B weights.

This is intentionally separate from `certify-model`: it runs deterministic
generation through the normal runtime CLI against local real checkpoints and
records auditable output.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import re
import subprocess
import sys
import time
from dataclasses import asdict, dataclass
from pathlib import Path


if hasattr(sys.stdout, "reconfigure"):
    sys.stdout.reconfigure(errors="backslashreplace")
if hasattr(sys.stderr, "reconfigure"):
    sys.stderr.reconfigure(errors="backslashreplace")


@dataclass
class RunConfig:
    name: str
    model_dir: str
    engine: str
    prompt_ids: str
    new_tokens: int
    cache_dir: str | None


@dataclass
class RunResult:
    name: str
    model_dir: str
    engine: str
    prompt_ids: str
    new_tokens: int
    command: list[str]
    exit_code: int
    elapsed_seconds: float
    stdout_sha256: str
    stderr_sha256: str
    output_ids: list[int]
    status_ok: bool
    stdout_tail: str
    stderr_tail: str


def default_binary() -> str:
    exe = "zymatica-engine.exe" if os.name == "nt" else "zymatica-engine"
    return str(Path("target") / "release" / exe)


def sha256_text(value: str) -> str:
    return hashlib.sha256(value.encode("utf-8", errors="replace")).hexdigest()


def tail_text(value: str, max_chars: int = 4000) -> str:
    return value[-max_chars:]


def parse_output_ids(stdout: str) -> list[int]:
    match = re.search(r"output_ids=\[([^\]]*)\]", stdout)
    if not match:
        return []
    raw = match.group(1).strip()
    if not raw:
        return []
    return [int(part.strip()) for part in raw.split(",") if part.strip()]


def prompt_id_count(prompt_ids: str) -> int:
    return len([part for part in prompt_ids.split(",") if part.strip()])


def run_one(binary: str, cfg: RunConfig, timeout_seconds: int) -> RunResult:
    command = [
        binary,
        "full-inference",
        "--model-dir",
        cfg.model_dir,
        "--prompt-ids",
        cfg.prompt_ids,
        "--new-tokens",
        str(cfg.new_tokens),
        "--engine",
        cfg.engine,
    ]
    if cfg.cache_dir:
        command.extend(["--q8-cache-dir", cfg.cache_dir])

    print(f"\n=== Running {cfg.name} full inference ===", flush=True)
    print("command=" + " ".join(command), flush=True)

    started = time.time()
    process = subprocess.Popen(
        command,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        text=True,
        encoding="utf-8",
        errors="backslashreplace",
    )

    while process.poll() is None:
        elapsed = time.time() - started
        if elapsed > timeout_seconds:
            process.kill()
            stdout, stderr = process.communicate()
            stderr += f"\nTIMEOUT after {timeout_seconds} seconds\n"
            break
        print(f"[{cfg.name}] still running elapsed={elapsed:.1f}s", flush=True)
        time.sleep(30)
    else:
        stdout, stderr = process.communicate()

    elapsed = time.time() - started
    exit_code = process.returncode if process.returncode is not None else -9
    output_ids = parse_output_ids(stdout)
    expected_output_ids = prompt_id_count(cfg.prompt_ids) + cfg.new_tokens
    status_ok = (
        exit_code == 0
        and "status=ok" in stdout
        and len(output_ids) == expected_output_ids
    )

    print(f"[{cfg.name}] exit_code={exit_code} elapsed={elapsed:.1f}s status_ok={status_ok}", flush=True)
    if output_ids:
        print(f"[{cfg.name}] output_ids={output_ids}", flush=True)
    if stderr.strip():
        print(f"[{cfg.name}] stderr_tail={tail_text(stderr, 1000)}", file=sys.stderr, flush=True)

    return RunResult(
        name=cfg.name,
        model_dir=cfg.model_dir,
        engine=cfg.engine,
        prompt_ids=cfg.prompt_ids,
        new_tokens=cfg.new_tokens,
        command=command,
        exit_code=exit_code,
        elapsed_seconds=elapsed,
        stdout_sha256=sha256_text(stdout),
        stderr_sha256=sha256_text(stderr),
        output_ids=output_ids,
        status_ok=status_ok,
        stdout_tail=tail_text(stdout),
        stderr_tail=tail_text(stderr),
    )


def write_reports(results: list[RunResult], out_json: Path, out_md: Path) -> None:
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_md.parent.mkdir(parents=True, exist_ok=True)

    payload = {
        "generated_at_unix": int(time.time()),
        "all_passed": all(result.status_ok for result in results),
        "results": [asdict(result) for result in results],
    }
    out_json.write_text(json.dumps(payload, indent=2), encoding="utf-8", newline="\n")

    lines = [
        "# Full Inference Validation Report",
        "",
        f"- Generated Unix time: `{payload['generated_at_unix']}`",
        f"- Overall status: `{'PASS' if payload['all_passed'] else 'FAIL'}`",
        "",
        "| Model | Engine | New tokens | Exit | Status | Elapsed seconds | Output token count |",
        "| --- | --- | ---: | ---: | --- | ---: | ---: |",
    ]
    for result in results:
        lines.append(
            f"| {result.name} | {result.engine} | {result.new_tokens} | {result.exit_code} | "
            f"{'PASS' if result.status_ok else 'FAIL'} | {result.elapsed_seconds:.1f} | {len(result.output_ids)} |"
        )

    lines.extend(["", "## Output IDs", ""])
    for result in results:
        lines.extend(
            [
                f"### {result.name}",
                "",
                f"- stdout sha256: `{result.stdout_sha256}`",
                f"- stderr sha256: `{result.stderr_sha256}`",
                f"- output ids: `{result.output_ids}`",
                "",
                "```text",
                result.stdout_tail.strip(),
                "```",
                "",
            ]
        )

    out_md.write_text("\n".join(lines) + "\n", encoding="utf-8", newline="\n")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--binary", default=default_binary())
    parser.add_argument("--e2b-dir", default=r"E:\models\gemma-4-E2B-it")
    parser.add_argument("--e4b-dir", default=r"E:\models\gemma-4-E4B-it")
    parser.add_argument("--e2b-engine", default="q4")
    parser.add_argument("--e4b-engine", default="f32")
    parser.add_argument("--e2b-cache-dir", default=r"E:\models\gemma-4-E2B-it\.zymatica-cache-q4")
    parser.add_argument("--e4b-cache-dir", default="")
    parser.add_argument("--prompt-ids", default="2")
    parser.add_argument("--new-tokens", type=int, default=16)
    parser.add_argument("--timeout-seconds", type=int, default=3600)
    parser.add_argument("--out-json", default="evidence/full_inference_validation.json")
    parser.add_argument("--out-md", default="full_inference_report.md")
    args = parser.parse_args()

    configs = [
        RunConfig(
            name="gemma-4-E2B-it",
            model_dir=args.e2b_dir,
            engine=args.e2b_engine,
            prompt_ids=args.prompt_ids,
            new_tokens=args.new_tokens,
            cache_dir=args.e2b_cache_dir or None,
        ),
        RunConfig(
            name="gemma-4-E4B-it",
            model_dir=args.e4b_dir,
            engine=args.e4b_engine,
            prompt_ids=args.prompt_ids,
            new_tokens=args.new_tokens,
            cache_dir=args.e4b_cache_dir or None,
        ),
    ]

    missing = [cfg.model_dir for cfg in configs if not Path(cfg.model_dir).exists()]
    if missing:
        print(f"Missing model directories: {missing}", file=sys.stderr)
        return 2

    if not Path(args.binary).exists():
        print(f"Engine binary does not exist: {args.binary}", file=sys.stderr)
        return 2

    results = [run_one(args.binary, cfg, args.timeout_seconds) for cfg in configs]
    write_reports(results, Path(args.out_json), Path(args.out_md))

    print(f"\nwrote_json={args.out_json}")
    print(f"wrote_markdown={args.out_md}")
    print(f"overall_status={'PASS' if all(result.status_ok for result in results) else 'FAIL'}")
    return 0 if all(result.status_ok for result in results) else 1


if __name__ == "__main__":
    raise SystemExit(main())
