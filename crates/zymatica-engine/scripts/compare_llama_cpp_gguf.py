#!/usr/bin/env python3
"""Compare Zymatica GGUF-import output against llama.cpp greedy generation.

This harness requires real local llama.cpp binaries and a real GGUF file. It
does not synthesize expected tokens: llama-cli generates text, llama-tokenize
converts the prompt plus generated text back to token IDs, and Zymatica is run
through the GGUF import cache path with the same prompt IDs.
"""

from __future__ import annotations

import argparse
import ast
import json
import os
import re
import shutil
import subprocess
import tempfile
from pathlib import Path


def run(command: list[str], cwd: str | None = None) -> str:
    completed = subprocess.run(
        command,
        cwd=cwd,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.STDOUT,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(
            f"command failed with exit code {completed.returncode}: {' '.join(command)}\n"
            f"{completed.stdout}"
        )
    return completed.stdout


def parse_tokenizer_ids(output: str) -> list[int]:
    ids: list[int] = []
    for line in output.splitlines():
        bracketed = re.search(r"\[\s*(-?\d+(?:\s*,\s*-?\d+)*)\s*\]", line)
        if bracketed:
            ids.extend(int(part.strip()) for part in bracketed.group(1).split(","))
            continue
        match = re.match(r"\s*(\d+)\s*(?:->|:|\t|\s)", line)
        if match:
            ids.append(int(match.group(1)))
    if ids:
        return ids
    plain_ints = [int(v) for v in re.findall(r"(?<![\w.-])-?\d+(?![\w.-])", output)]
    if plain_ints:
        return plain_ints
    raise RuntimeError("could not parse token IDs from llama-tokenize output\n" + output)


def parse_zymatica_ids(stdout: str) -> list[int]:
    for line in stdout.splitlines():
        if line.startswith("output_ids="):
            return [int(v) for v in ast.literal_eval(line.split("=", 1)[1])]
    raise RuntimeError("Zymatica output did not contain output_ids= line\n" + stdout)


def llama_tokenize(llama_tokenize_bin: str, gguf: str, text: str) -> list[int]:
    output = run([llama_tokenize_bin, "-m", gguf, "--text", text])
    return parse_tokenizer_ids(output)


def llama_generate(llama_cli: str, gguf: str, prompt: str, new_tokens: int) -> str:
    output = run(
        [
            llama_cli,
            "-m",
            gguf,
            "-p",
            prompt,
            "-n",
            str(new_tokens),
            "--temp",
            "0",
            "--top-k",
            "1",
            "--seed",
            "0",
            "--no-display-prompt",
        ]
    )
    return output.strip()


def zymatica_import(
    binary: str,
    repo_dir: str,
    gguf: str,
    model_dir: str,
    cache_dir: str,
    mode: str,
) -> None:
    run(
        [
            binary,
            "gguf-import",
            "--gguf",
            gguf,
            "--model-dir",
            model_dir,
            "--cache-dir",
            cache_dir,
            "--mode",
            mode,
        ],
        cwd=repo_dir,
    )


def zymatica_generate(
    binary: str,
    repo_dir: str,
    model_dir: str,
    cache_dir: str,
    mode: str,
    prompt_ids: list[int],
    new_tokens: int,
) -> list[int]:
    stdout = run(
        [
            binary,
            "full-inference",
            "--model-dir",
            model_dir,
            "--prompt-ids",
            ",".join(str(v) for v in prompt_ids),
            "--new-tokens",
            str(new_tokens),
            "--engine",
            mode,
            "--q8-cache-dir",
            cache_dir,
        ],
        cwd=repo_dir,
    )
    return parse_zymatica_ids(stdout)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--gguf", required=True)
    parser.add_argument("--model-dir", required=True, help="HF config/tokenizer sidecar directory")
    parser.add_argument("--prompt", required=True)
    parser.add_argument("--new-tokens", type=int, default=32)
    parser.add_argument("--mode", choices=["q8", "q5", "q4"], default="q8")
    parser.add_argument("--cache-dir")
    parser.add_argument("--repo-dir", default=str(Path(__file__).resolve().parents[1]))
    parser.add_argument("--binary", default="target/release/zymatica-engine.exe")
    parser.add_argument("--llama-cli", default="llama-cli")
    parser.add_argument("--llama-tokenize", default="llama-tokenize")
    parser.add_argument("--skip-import", action="store_true")
    parser.add_argument("--json", action="store_true")
    args = parser.parse_args()

    llama_cli = shutil.which(args.llama_cli) or args.llama_cli
    llama_tokenize_bin = shutil.which(args.llama_tokenize) or args.llama_tokenize
    binary = str(Path(args.binary))
    if not Path(binary).is_absolute():
        binary = str(Path(args.repo_dir) / binary)

    cache_temp = None
    cache_dir = args.cache_dir
    if cache_dir is None:
        cache_temp = tempfile.TemporaryDirectory(prefix="zymatica-gguf-parity-")
        cache_dir = cache_temp.name

    try:
        prompt_ids = llama_tokenize(llama_tokenize_bin, args.gguf, args.prompt)
        llama_generated_text = llama_generate(
            llama_cli, args.gguf, args.prompt, args.new_tokens
        )
        llama_full_ids = llama_tokenize(
            llama_tokenize_bin, args.gguf, args.prompt + llama_generated_text
        )
        llama_ids = llama_full_ids[: len(prompt_ids) + args.new_tokens]

        if not args.skip_import:
            zymatica_import(
                binary,
                args.repo_dir,
                args.gguf,
                args.model_dir,
                cache_dir,
                args.mode,
            )
        zymatica_ids = zymatica_generate(
            binary,
            args.repo_dir,
            args.model_dir,
            cache_dir,
            args.mode,
            prompt_ids,
            args.new_tokens,
        )

        matched = llama_ids == zymatica_ids
        report = {
            "gguf": os.path.abspath(args.gguf),
            "model_dir": os.path.abspath(args.model_dir),
            "mode": args.mode,
            "prompt_ids": prompt_ids,
            "llama_ids": llama_ids,
            "zymatica_ids": zymatica_ids,
            "matched": matched,
        }
        if args.json:
            print(json.dumps(report, indent=2))
        else:
            print(f"mode={args.mode}")
            print(f"prompt_ids={prompt_ids}")
            print(f"llama_ids={llama_ids}")
            print(f"zymatica_ids={zymatica_ids}")
            print(f"matched={matched}")
            if not matched:
                for idx, (llama_id, zymatica_id) in enumerate(
                    zip(llama_ids, zymatica_ids)
                ):
                    if llama_id != zymatica_id:
                        print(
                            "first_mismatch_index="
                            f"{idx} llama={llama_id} zymatica={zymatica_id}"
                        )
                        break
        return 0 if matched else 1
    finally:
        if cache_temp is not None:
            cache_temp.cleanup()


if __name__ == "__main__":
    raise SystemExit(main())
