#!/usr/bin/env python3
"""Compare Zymatica greedy output against Hugging Face for a fixed token prompt.

This is a certification harness, not part of the embedded Pi runtime. It loads
the HF checkpoint with transformers, runs deterministic greedy generation, runs
the native Zymatica binary, and requires exact token-id equality.
"""

from __future__ import annotations

import argparse
import ast
import json
import os
import subprocess
import sys
from pathlib import Path


def parse_ids(value: str) -> list[int]:
    return [int(part.strip()) for part in value.split(",") if part.strip()]


def parse_checkpoints(value: str | None, max_new_tokens: int) -> list[int]:
    if not value:
        return [max_new_tokens]
    checkpoints = sorted(set(parse_ids(value)))
    invalid = [v for v in checkpoints if v <= 0 or v > max_new_tokens]
    if invalid:
        raise ValueError(
            f"checkpoints must be in range 1..{max_new_tokens}; invalid={invalid}"
        )
    return checkpoints


def format_ids(ids: list[int], limit: int = 80) -> str:
    if len(ids) <= limit:
        return repr(ids)
    head = ids[:40]
    tail = ids[-10:]
    return f"{head!r} ... {tail!r} (len={len(ids)})"


def torch_dtype(name: str):
    import torch

    match name:
        case "bfloat16":
            return torch.bfloat16
        case "float16":
            return torch.float16
        case "float32":
            return torch.float32
        case _:
            raise ValueError(f"unsupported dtype: {name}")


def first_model_device(model):
    import torch

    for parameter in model.parameters():
        if parameter.device.type != "meta":
            return parameter.device
    return torch.device("cpu")


def load_hf_model(args: argparse.Namespace):
    import torch
    from transformers import AutoConfig, AutoModelForCausalLM, Gemma4ForCausalLM

    # Some tiny compatibility fixtures intentionally omit newer Gemma4-only
    # tensors so Transformers initializes them at load time. Seed that path so
    # reference comparisons are deterministic across separate q4/q5/q8 runs.
    torch.manual_seed(42)
    if args.hf_device == "auto":
        device = "cuda" if torch.cuda.is_available() else "cpu"
    else:
        device = args.hf_device
    kwargs = {
        "local_files_only": True,
        "dtype": torch_dtype(args.hf_dtype),
    }
    if args.hf_device_map != "none":
        kwargs["device_map"] = args.hf_device_map
        max_memory = {}
        if args.hf_max_gpu_memory:
            max_memory[0] = args.hf_max_gpu_memory
        if args.hf_max_cpu_memory:
            max_memory["cpu"] = args.hf_max_cpu_memory
        if max_memory:
            kwargs["max_memory"] = max_memory

    if args.hf_model_kind == "gemma4-text":
        config = AutoConfig.from_pretrained(args.model_dir, local_files_only=True)
        if hasattr(config, "text_config"):
            config = config.text_config
        model = Gemma4ForCausalLM.from_pretrained(args.model_dir, config=config, **kwargs)
    else:
        model = AutoModelForCausalLM.from_pretrained(args.model_dir, **kwargs)
    if args.hf_device_map == "none":
        model.to(device)
    model.eval()
    return model, first_model_device(model)


def hf_greedy(
    args: argparse.Namespace, prompt_ids: list[int], new_tokens: int, use_cache: bool
) -> list[int]:
    import torch

    model, device = load_hf_model(args)
    output = list(prompt_ids)

    if use_cache:
        with torch.no_grad():
            input_ids = torch.tensor([prompt_ids], dtype=torch.long, device=device)
            attention_mask = torch.ones_like(input_ids, device=device)
            model_kwargs = {
                "input_ids": input_ids,
                "attention_mask": attention_mask,
                "use_cache": True,
            }
            if supports_kwarg(model.forward, "cache_position"):
                model_kwargs["cache_position"] = torch.arange(
                    0, input_ids.shape[1], dtype=torch.long, device=device
                )
            outputs = model(**model_kwargs)
            past_key_values = outputs.past_key_values

            for step in range(new_tokens):
                next_token_logits = outputs.logits[0, -1, :]
                next_token = int(torch.argmax(next_token_logits).item())
                output.append(next_token)
                if step + 1 == new_tokens:
                    break
                position = len(output) - 1
                input_ids = torch.tensor([[next_token]], dtype=torch.long, device=device)
                attention_mask = torch.ones((1, len(output)), dtype=torch.long, device=device)
                model_kwargs = {
                    "input_ids": input_ids,
                    "attention_mask": attention_mask,
                    "past_key_values": past_key_values,
                    "use_cache": True,
                }
                if supports_kwarg(model.forward, "cache_position"):
                    model_kwargs["cache_position"] = torch.tensor(
                        [position], dtype=torch.long, device=device
                    )
                outputs = model(**model_kwargs)
                past_key_values = outputs.past_key_values
                if args.hf_progress_every and (step + 1) % args.hf_progress_every == 0:
                    print(
                        f"hf_cached_step={step + 1}/{new_tokens}",
                        file=sys.stderr,
                        flush=True,
                    )
    else:
        input_ids = torch.tensor([prompt_ids], dtype=torch.long, device=device)
        for step in range(new_tokens):
            with torch.no_grad():
                outputs = model(input_ids)
                next_token_logits = outputs.logits[0, -1, :]
                next_token = int(torch.argmax(next_token_logits).item())
                output.append(next_token)
                next_input = torch.tensor([[next_token]], device=device)
                input_ids = torch.cat([input_ids, next_input], dim=-1)
                if args.hf_progress_every and (step + 1) % args.hf_progress_every == 0:
                    print(
                        f"hf_full_step={step + 1}/{new_tokens}",
                        file=sys.stderr,
                        flush=True,
                    )
    return output


def supports_kwarg(callable_obj, name: str) -> bool:
    import inspect

    try:
        signature = inspect.signature(callable_obj)
    except (TypeError, ValueError):
        return False
    return name in signature.parameters


def parse_zymatica_ids(stdout: str) -> list[int]:
    for line in stdout.splitlines():
        if line.startswith("output_ids="):
            return [int(v) for v in ast.literal_eval(line.split("=", 1)[1])]
    raise RuntimeError("Zymatica output did not contain output_ids= line\n" + stdout)


def zymatica_greedy(args: argparse.Namespace) -> tuple[list[int], str]:
    if args.binary:
        command = [
            str(Path(args.binary)),
            "full-inference",
        ]
    else:
        command = [
            "cargo",
            "run",
            "--release",
            "--",
            "full-inference",
        ]
    command.extend(
        [
            "--model-dir",
            args.model_dir,
            "--prompt-ids",
            args.prompt_ids,
            "--new-tokens",
            str(args.new_tokens),
            "--engine",
            args.engine,
        ]
    )
    if args.q8_cache_dir:
        command.extend(["--q8-cache-dir", args.q8_cache_dir])
    completed = subprocess.run(
        command,
        cwd=args.repo_dir,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"Zymatica command failed with exit code {completed.returncode}\n{completed.stdout}"
        )
    return parse_zymatica_ids(completed.stdout), completed.stdout


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--model-dir", required=True)
    parser.add_argument("--repo-dir", default=str(Path(__file__).resolve().parents[1]))
    parser.add_argument("--prompt-ids", default="2")
    parser.add_argument("--new-tokens", type=int, default=32)
    parser.add_argument(
        "--checkpoints",
        help=(
            "Comma-separated new-token checkpoints to validate from one full generation, "
            "for example 32,64,128,256,512,1024,2048,3264."
        ),
    )
    parser.add_argument("--engine", choices=["f32", "q8", "q5", "q4"], default="f32")
    parser.add_argument("--q8-cache-dir")
    parser.add_argument("--binary")
    parser.add_argument(
        "--hf-model-kind",
        choices=["auto", "gemma4-text"],
        default="auto",
        help="HF model loader. gemma4-text expects a text-only Gemma4 checkpoint.",
    )
    parser.add_argument(
        "--hf-dtype",
        choices=["bfloat16", "float16", "float32"],
        default="bfloat16",
    )
    parser.add_argument("--hf-device", choices=["auto", "cpu", "cuda"], default="auto")
    parser.add_argument(
        "--hf-device-map",
        default="none",
        help="Pass an Accelerate device_map value such as auto; default disables device_map.",
    )
    parser.add_argument("--hf-max-gpu-memory")
    parser.add_argument("--hf-max-cpu-memory")
    parser.add_argument("--hf-progress-every", type=int, default=0)
    parser.add_argument("--json", action="store_true")
    parser.add_argument(
        "--hf-use-cache",
        action="store_true",
        help=(
            "Use the Hugging Face KV cache path with explicit attention_mask/cache_position "
            "for long-context reference speed."
        ),
    )
    parser.add_argument(
        "--hf-cache-self-check",
        action="store_true",
        help=(
            "When --hf-use-cache is set, also run the slower full-recompute HF path and "
            "require equality at all requested checkpoints."
        ),
    )
    args = parser.parse_args()

    prompt_ids = parse_ids(args.prompt_ids)
    checkpoints = parse_checkpoints(args.checkpoints, args.new_tokens)
    print("hf_reference_start", file=sys.stderr, flush=True)
    hf_ids = hf_greedy(args, prompt_ids, args.new_tokens, args.hf_use_cache)
    print("hf_reference_done", file=sys.stderr, flush=True)
    hf_full_ids = None
    if args.hf_use_cache and args.hf_cache_self_check:
        print("hf_full_self_check_start", file=sys.stderr, flush=True)
        hf_full_ids = hf_greedy(args, prompt_ids, args.new_tokens, False)
        print("hf_full_self_check_done", file=sys.stderr, flush=True)
    print("zymatica_start", file=sys.stderr, flush=True)
    zymatica_ids, zymatica_stdout = zymatica_greedy(args)
    print("zymatica_done", file=sys.stderr, flush=True)
    matched = hf_ids == zymatica_ids
    checkpoint_matches = {}
    for checkpoint in checkpoints:
        end = len(prompt_ids) + checkpoint
        checkpoint_matches[str(checkpoint)] = hf_ids[:end] == zymatica_ids[:end]
    hf_cache_checkpoint_matches = {}
    if hf_full_ids is not None:
        for checkpoint in checkpoints:
            end = len(prompt_ids) + checkpoint
            hf_cache_checkpoint_matches[str(checkpoint)] = hf_ids[:end] == hf_full_ids[:end]
    report = {
        "model_dir": os.path.abspath(args.model_dir),
        "engine": args.engine,
        "prompt_ids": prompt_ids,
        "new_tokens": args.new_tokens,
        "checkpoints": checkpoints,
        "hf_ids": hf_ids,
        "zymatica_ids": zymatica_ids,
        "matched": matched,
        "checkpoint_matches": checkpoint_matches,
        "hf_cache_self_check": hf_full_ids is not None,
        "hf_cache_checkpoint_matches": hf_cache_checkpoint_matches,
    }
    if args.json:
        print(json.dumps(report, indent=2))
    else:
        print(f"engine={args.engine}")
        print(f"prompt_ids={prompt_ids}")
        print(f"new_tokens={args.new_tokens}")
        print(f"checkpoints={checkpoints}")
        print(f"hf_ids={format_ids(hf_ids)}")
        print(f"zymatica_ids={format_ids(zymatica_ids)}")
        for checkpoint, ok in checkpoint_matches.items():
            print(f"checkpoint_{checkpoint}_matched={ok}")
        for checkpoint, ok in hf_cache_checkpoint_matches.items():
            print(f"hf_cache_checkpoint_{checkpoint}_matched={ok}")
        print(f"matched={matched}")
    if hf_full_ids is not None and hf_ids != hf_full_ids:
        print("hf_cache_self_check_matched=False")
        for idx, (cached_id, full_id) in enumerate(zip(hf_ids, hf_full_ids)):
            if cached_id != full_id:
                print(
                    f"hf_cache_first_mismatch_index={idx} cached={cached_id} full={full_id}"
                )
                break
        return 1
    if not matched:
        for idx, (hf_id, zy_id) in enumerate(zip(hf_ids, zymatica_ids)):
            if hf_id != zy_id:
                print(f"first_mismatch_index={idx} hf={hf_id} zymatica={zy_id}")
                break
        else:
            print(f"length_mismatch hf={len(hf_ids)} zymatica={len(zymatica_ids)}")
        print("\n--- zymatica stdout ---")
        print(zymatica_stdout)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
