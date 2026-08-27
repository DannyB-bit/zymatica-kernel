#!/usr/bin/env python3
"""Derive a Gemma4 text-only HF checkpoint from a real multimodal checkpoint."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
from pathlib import Path

from safetensors import safe_open
from safetensors.torch import save_file


LANGUAGE_PREFIX = "model.language_model."


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--source-model-dir", required=True)
    parser.add_argument("--output-dir", required=True)
    parser.add_argument("--force", action="store_true")
    args = parser.parse_args()

    source = Path(args.source_model_dir)
    output = Path(args.output_dir)
    source_config_path = source / "config.json"
    source_weights_path = source / "model.safetensors"
    if not source_config_path.is_file():
        raise SystemExit(f"source config not found: {source_config_path}")
    if not source_weights_path.is_file():
        raise SystemExit(f"source safetensors not found: {source_weights_path}")
    if output.exists() and any(output.iterdir()) and not args.force:
        raise SystemExit(f"output directory is not empty: {output}")
    output.mkdir(parents=True, exist_ok=True)

    config = json.loads(source_config_path.read_text(encoding="utf-8"))
    text_config = dict(config.get("text_config") or {})
    if not text_config:
        raise SystemExit("source config does not contain text_config")
    text_config["architectures"] = ["Gemma4ForCausalLM"]
    text_config["model_type"] = "gemma4_text"
    (output / "config.json").write_text(
        json.dumps(text_config, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    for name in [
        "tokenizer.json",
        "tokenizer_config.json",
        "chat_template.jinja",
        "generation_config.json",
    ]:
        src = source / name
        if src.exists():
            shutil.copy2(src, output / name)

    tensors = {}
    source_key_count = 0
    with safe_open(str(source_weights_path), framework="pt", device="cpu") as handle:
        for key in handle.keys():
            if key.startswith(LANGUAGE_PREFIX):
                source_key_count += 1
                tensors["model." + key[len(LANGUAGE_PREFIX) :]] = handle.get_tensor(key)
    if not tensors:
        raise SystemExit(f"no tensors found with prefix {LANGUAGE_PREFIX!r}")

    output_weights_path = output / "model.safetensors"
    save_file(
        tensors,
        str(output_weights_path),
        metadata={
            "format": "pt",
            "source_model_dir": str(source.resolve()),
            "source_weight_sha256": sha256_file(source_weights_path),
            "key_transform": f"strip {LANGUAGE_PREFIX}",
        },
    )

    manifest = {
        "source_model_dir": str(source.resolve()),
        "source_weights": str(source_weights_path.resolve()),
        "source_weight_sha256": sha256_file(source_weights_path),
        "output_weights": str(output_weights_path.resolve()),
        "output_weight_sha256": sha256_file(output_weights_path),
        "source_language_tensor_count": source_key_count,
        "output_tensor_count": len(tensors),
        "key_transform": f"{LANGUAGE_PREFIX}* -> model.*",
    }
    (output / "zymatica_text_checkpoint_manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    print(json.dumps(manifest, indent=2, sort_keys=True))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
