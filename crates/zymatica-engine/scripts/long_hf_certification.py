#!/usr/bin/env python3
"""Run long Hugging Face parity certification against real local artifacts."""

from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-dir", required=True)
    parser.add_argument("--binary", required=True)
    parser.add_argument("--engine", default="f32", choices=["f32", "q8", "q5", "q4"])
    parser.add_argument("--new-tokens", type=int, default=3264)
    parser.add_argument(
        "--checkpoints",
        default="32,64,128,256,512,1024,2048,3264",
    )
    parser.add_argument(
        "--prompt-ids",
        nargs="+",
        default=["2", "1,2,3", "2,236761,108"],
        help="One or more comma-separated prompt id sequences.",
    )
    parser.add_argument("--hf-use-cache", action="store_true")
    parser.add_argument("--hf-cache-self-check", action="store_true")
    parser.add_argument("--hf-model-kind", choices=["auto", "gemma4-text"], default="auto")
    parser.add_argument(
        "--hf-dtype",
        choices=["bfloat16", "float16", "float32"],
        default="bfloat16",
    )
    parser.add_argument("--hf-device", choices=["auto", "cpu", "cuda"], default="auto")
    parser.add_argument("--hf-device-map", default="none")
    parser.add_argument("--hf-max-gpu-memory")
    parser.add_argument("--hf-max-cpu-memory")
    parser.add_argument("--hf-progress-every", type=int, default=0)
    args = parser.parse_args()

    model_dir = Path(args.model_dir)
    binary = Path(args.binary)
    if not model_dir.is_dir():
        raise SystemExit(f"model directory not found: {model_dir}")
    if not binary.is_file():
        raise SystemExit(f"zymatica binary not found: {binary}")

    harness = Path(__file__).with_name("compare_hf_reference.py")
    if not harness.is_file():
        raise SystemExit(f"HF comparison harness not found: {harness}")

    for prompt_ids in args.prompt_ids:
        cmd = [
            sys.executable,
            str(harness),
            "--model-dir",
            str(model_dir),
            "--prompt-ids",
            prompt_ids,
            "--new-tokens",
            str(args.new_tokens),
            "--checkpoints",
            args.checkpoints,
            "--engine",
            args.engine,
            "--binary",
            str(binary),
        ]
        if args.hf_use_cache:
            cmd.append("--hf-use-cache")
        if args.hf_cache_self_check:
            cmd.append("--hf-cache-self-check")
        cmd.extend(["--hf-model-kind", args.hf_model_kind])
        cmd.extend(["--hf-dtype", args.hf_dtype])
        cmd.extend(["--hf-device", args.hf_device])
        cmd.extend(["--hf-device-map", args.hf_device_map])
        if args.hf_max_gpu_memory:
            cmd.extend(["--hf-max-gpu-memory", args.hf_max_gpu_memory])
        if args.hf_max_cpu_memory:
            cmd.extend(["--hf-max-cpu-memory", args.hf_max_cpu_memory])
        if args.hf_progress_every:
            cmd.extend(["--hf-progress-every", str(args.hf_progress_every)])
        print("running:", " ".join(cmd), flush=True)
        subprocess.run(cmd, check=True)

    print("long_hf_certification=passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
