#!/usr/bin/env python3
"""Run real full model inference benchmarks on E2B, E4B, and Qwen 3.5.
Gathers execution timing, token throughput (tokens/sec), output token IDs, and exit status.
"""

from __future__ import annotations
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
class BenchTask:
    name: str
    model_dir: str
    engine: str
    prompt_ids: str
    new_tokens: int
    cache_dir: str | None = None

@dataclass
class BenchResult:
    name: str
    model_dir: str
    engine: str
    prompt_ids: str
    new_tokens: int
    exit_code: int
    elapsed_seconds: float
    output_ids: list[int]
    tokens_per_second: float
    status_ok: bool
    stdout_tail: str
    stderr_tail: str

def get_binary() -> str:
    exe = "zymatica-engine.exe" if os.name == "nt" else "zymatica-engine"
    return str(Path("target") / "release" / exe)

def parse_output_ids(stdout: str) -> list[int]:
    match = re.search(r"output_ids=\[([^\]]*)\]", stdout)
    if not match:
        return []
    raw = match.group(1).strip()
    if not raw:
        return []
    return [int(part.strip()) for part in raw.split(",") if part.strip()]

def run_bench(binary: str, task: BenchTask) -> BenchResult:
    cmd = [
        binary,
        "full-inference",
        "--model-dir", task.model_dir,
        "--prompt-ids", task.prompt_ids,
        "--new-tokens", str(task.new_tokens),
        "--engine", task.engine,
    ]
    if task.cache_dir:
        cmd.extend(["--q8-cache-dir", task.cache_dir])

    print(f"\n==========================================", flush=True)
    print(f"BENCHMARK: {task.name} ({task.engine} engine)", flush=True)
    print(f"Model Dir: {task.model_dir}", flush=True)
    print(f"Command: {' '.join(cmd)}", flush=True)
    print(f"==========================================", flush=True)

    t0 = time.time()
    proc = subprocess.run(
        cmd,
        capture_output=True,
        text=True,
        encoding="utf-8",
        errors="backslashreplace",
    )
    elapsed_total = time.time() - t0

    output_ids = parse_output_ids(proc.stdout)
    status_ok = proc.returncode == 0 and ("status=ok" in proc.stdout or len(output_ids) > 0)
    
    elapsed_gen_ms = None
    gen_match = re.search(r"elapsed_ms=([\d\.]+)", proc.stdout)
    if gen_match:
        elapsed_gen_ms = float(gen_match.group(1))

    if elapsed_gen_ms and elapsed_gen_ms > 0:
        gen_seconds = elapsed_gen_ms / 1000.0
        tok_per_sec = task.new_tokens / gen_seconds
    else:
        gen_seconds = elapsed_total
        tok_per_sec = (task.new_tokens / elapsed_total) if elapsed_total > 0 else 0.0

    print(f"Exit Code: {proc.returncode}")
    print(f"Total Process Time: {elapsed_total:.2f}s")
    if elapsed_gen_ms is not None:
        print(f"Token Generation Time: {gen_seconds:.2f}s ({elapsed_gen_ms:.1f} ms)")
    print(f"Output IDs: {output_ids}")
    print(f"Decode Throughput: {tok_per_sec:.2f} tokens/sec")
    print(f"Status: {'PASS' if status_ok else 'FAIL'}")

    return BenchResult(
        name=task.name,
        model_dir=task.model_dir,
        engine=task.engine,
        prompt_ids=task.prompt_ids,
        new_tokens=task.new_tokens,
        exit_code=proc.returncode,
        elapsed_seconds=gen_seconds,
        output_ids=output_ids,
        tokens_per_second=tok_per_sec,
        status_ok=status_ok,
        stdout_tail=proc.stdout[-2000:],
        stderr_tail=proc.stderr[-1000:],
    )

def main():
    binary = get_binary()
    if not Path(binary).exists():
        print(f"Error: binary {binary} not found. Run cargo build --release first.", file=sys.stderr)
        return 1

    tasks = [
        BenchTask(
            name="Gemma-4-E2B-it (Q4 Resident RAM Engine)",
            model_dir=r"E:\models\gemma-4-E2B-it",
            engine="q4",
            prompt_ids="2",
            new_tokens=16,
            cache_dir=None,
        ),
        BenchTask(
            name="Gemma-4-E2B-it (Q8 Mmap Cache Engine)",
            model_dir=r"E:\models\gemma-4-E2B-it",
            engine="q8",
            prompt_ids="2",
            new_tokens=16,
            cache_dir=r"E:\models\gemma-4-E2B-it\.zymatica-cache-q8",
        ),
        BenchTask(
            name="Gemma-4-E4B-it (F32 Full Engine)",
            model_dir=r"E:\models\gemma-4-E4B-it",
            engine="f32",
            prompt_ids="2",
            new_tokens=16,
            cache_dir=None,
        ),
        BenchTask(
            name="Qwen-3.5-0.8B (F32 Engine)",
            model_dir=r"E:\experiments\probe_qwen\qwen-3.5-0.8b-ufo",
            engine="f32",
            prompt_ids="151644",
            new_tokens=16,
        ),
    ]

    results = []
    for task in tasks:
        if not Path(task.model_dir).exists():
            print(f"Skipping {task.name}: model_dir {task.model_dir} does not exist.")
            continue
        res = run_bench(binary, task)
        results.append(res)

    out_json = Path("evidence/real_model_inference_benchmarks.json")
    out_json.parent.mkdir(parents=True, exist_ok=True)
    out_json.write_text(json.dumps([asdict(r) for r in results], indent=2), encoding="utf-8")

    print("\n\n==========================================")
    print("FINAL INFERENCE BENCHMARK SUMMARY")
    print("==========================================")
    print(f"{'Model / Engine':<32} | {'Tokens':<6} | {'Time (s)':<8} | {'Tokens/s':<8} | {'Status'}")
    print("-" * 70)
    for r in results:
        print(f"{r.name:<32} | {r.new_tokens:<6} | {r.elapsed_seconds:<8.2f} | {r.tokens_per_second:<8.2f} | {'PASS' if r.status_ok else 'FAIL'}")
    print("==========================================\n")

if __name__ == "__main__":
    sys.exit(main())
