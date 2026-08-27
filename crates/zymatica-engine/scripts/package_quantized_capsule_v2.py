import os
import gc
import json
import time
import argparse
import hashlib
import struct
import zipfile
import torch
import numpy as np
from safetensors import safe_open
from safetensors.torch import save_file

def quantize_q8(W):
    """Quantize to ZQ8: Magic (8B) + Rows (8B) + Cols (8B) + Scales (rows*4B) + Data (rows*cols i8)"""
    W = np.asarray(W, dtype=np.float32)
    rows, cols = W.shape
    max_abs = np.max(np.abs(W), axis=1)
    scales = np.where(max_abs > 0, max_abs / 127.0, 1.0).astype(np.float32)
    q = np.clip(np.rint(W / scales[:, None]), -127, 127).astype(np.int8)
    header = b"ZQ8M0001" + struct.pack("<QQ", rows, cols)
    return header + scales.tobytes() + q.tobytes()

def quantize_q4(W):
    """Quantize to ZQ4: Magic (8B) + Rows (8B) + Cols (8B) + Scales (rows*4B) + Data (rows*packed_cols u8)"""
    W = np.asarray(W, dtype=np.float32)
    rows, cols = W.shape
    max_abs = np.max(np.abs(W), axis=1)
    scales = np.where(max_abs > 0, max_abs / 7.0, 1.0).astype(np.float32)
    q = np.clip(np.rint(W / scales[:, None]), -7, 7).astype(np.int16)
    codes = (q + 8).astype(np.uint8)
    if cols % 2:
        codes = np.pad(codes, ((0, 0), (0, 1)), mode="constant", constant_values=8)
    packed = (codes[:, 0::2] | (codes[:, 1::2] << 4)).astype(np.uint8)
    header = b"ZQ4M0001" + struct.pack("<QQ", rows, cols)
    return header + scales.tobytes() + packed.tobytes()

def quantize_q5(W):
    """Quantize to ZQ5: Magic (8B) + Rows (8B) + Cols (8B) + Scales (rows*4B) + Bit-packed Data"""
    W = np.asarray(W, dtype=np.float32)
    rows, cols = W.shape
    max_abs = np.max(np.abs(W), axis=1)
    scales = np.where(max_abs > 0, max_abs / 15.0, 1.0).astype(np.float32)
    q = np.clip(np.rint(W / scales[:, None]), -15, 15).astype(np.int16)
    codes = (q + 16).astype(np.uint8).reshape(-1)
    packed = pack_5bit_codes(codes)
    header = b"ZQ5M0001" + struct.pack("<QQ", rows, cols)
    return header + scales.tobytes() + packed

def pack_5bit_codes(codes):
    original_len = int(codes.size)
    padded_len = ((original_len + 7) // 8) * 8
    if padded_len != original_len:
        codes = np.pad(codes, (0, padded_len - original_len), mode="constant", constant_values=16)
    groups = codes.reshape(-1, 8).astype(np.uint16)
    out = np.empty(groups.shape[0] * 5, dtype=np.uint8)
    out[0::5] = (groups[:, 0] | ((groups[:, 1] & 0x07) << 5)).astype(np.uint8)
    out[1::5] = (((groups[:, 1] >> 3) & 0x03) | (groups[:, 2] << 2) | ((groups[:, 3] & 0x01) << 7)).astype(np.uint8)
    out[2::5] = (((groups[:, 3] >> 1) & 0x0F) | ((groups[:, 4] & 0x0F) << 4)).astype(np.uint8)
    out[3::5] = (((groups[:, 4] >> 4) & 0x01) | (groups[:, 5] << 1) | ((groups[:, 6] & 0x03) << 6)).astype(np.uint8)
    out[4::5] = (((groups[:, 6] >> 2) & 0x07) | (groups[:, 7] << 3)).astype(np.uint8)
    return out[: (original_len * 5 + 7) // 8].tobytes()

def clip_round(val, min_val, max_val):
    r = round(val)
    if r < min_val:
        return min_val
    if r > max_val:
        return max_val
    return r

def file_sha256(path):
    digest = hashlib.sha256()
    with open(path, "rb") as fh:
        for chunk in iter(lambda: fh.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()

def manifest_file(path, archive_name):
    return {
        "path": archive_name,
        "archive_name": archive_name,
        "transform": "raw",
        "original_size": os.path.getsize(path),
        "original_sha256": file_sha256(path)
    }

def should_quantize_tensor(name, tensor, quantized_roles):
    if tensor.ndim != 2:
        return False
    # The engine currently reads the per-layer embedding table through
    # LazyRowTensor rather than QuantMatrix, so keep it in safetensors until
    # quantized lazy row tables are implemented.
    if "embed_tokens_per_layer" in name or "token_embedding_per_layer" in name:
        return False
    return any(role in name for role in quantized_roles)

def main():
    parser = argparse.ArgumentParser(description="Gemma Executable Quantized UFO Capsule v2 Packager")
    parser.add_argument("--model-dir", type=str, required=True, help="Directory containing unquantized safetensors")
    parser.add_argument("--out-capsule", type=str, required=True, help="Output .ufomodel.zip path")
    parser.add_argument("--mode", type=str, choices=["q4", "q5", "q8"], required=True, help="Quantization mode")
    args = parser.parse_args()

    print("=" * 80)
    print(f"  PACKAGING UFO CAPSULE V2 (Target: {args.mode.upper()})")
    print("=" * 80)

    # 1. Load config
    config_path = os.path.join(args.model_dir, "config.json")
    if not os.path.exists(config_path):
        print(f"[-] Error: config.json not found in {args.model_dir}")
        return
        
    with open(config_path, "r") as f:
        config_data = json.load(f)
        
    model_name = config_data.get("model_name", "gemma-quantized")

    # 2. Find safetensors files
    single_path = os.path.join(args.model_dir, "model.safetensors")
    index_path = os.path.join(args.model_dir, "model.safetensors.index.json")

    if os.path.exists(index_path):
        with open(index_path, "r") as f:
            index_data = json.load(f)
        weight_map = index_data.get("weight_map", {})
        shards = {}
        for param_name, shard_name in weight_map.items():
            if shard_name not in shards:
                shards[shard_name] = []
            shards[shard_name].append(param_name)
    elif os.path.exists(single_path):
        with safe_open(single_path, framework="pt", device="cpu") as f:
            keys = list(f.keys())
        shards = {"model.safetensors": keys}
    else:
        print(f"[-] Error: No safetensors files found in {args.model_dir}")
        return

    # 3. Process tensors
    quantized_files = {}
    unquantized_tensors = {}
    
    quantized_roles = [
        "q_proj", "k_proj", "v_proj", "o_proj",
        "gate_proj", "up_proj", "down_proj",
        "embed_tokens", "lm_head",
        "per_layer_model_projection",
        "per_layer_input_gate",
        "per_layer_projection",
    ]

    out_parent = os.path.dirname(os.path.abspath(args.out_capsule)) or os.getcwd()
    os.makedirs(out_parent, exist_ok=True)
    tmp_dir = os.path.join(
        out_parent,
        f".{os.path.basename(args.out_capsule)}.tmp_quant_v2_{int(time.time())}",
    )
    os.makedirs(tmp_dir, exist_ok=True)

    for shard_name, param_names in shards.items():
        shard_path = os.path.join(args.model_dir, shard_name)
        with safe_open(shard_path, framework="pt", device="cpu") as f:
            for name in param_names:
                W = f.get_tensor(name)
                
                should_quantize = should_quantize_tensor(name, W, quantized_roles)
                
                if should_quantize:
                    print(f"[*] Quantizing {name} ({W.shape[0]}x{W.shape[1]})...", flush=True)
                    W_np = W.float().numpy()
                    
                    if args.mode == "q8":
                        payload = quantize_q8(W_np)
                        ext = "zq8"
                    elif args.mode == "q5":
                        payload = quantize_q5(W_np)
                        ext = "zq5"
                    else:
                        payload = quantize_q4(W_np)
                        ext = "zq4"
                        
                    filename = f"{name}.{ext}"
                    filepath = os.path.join(tmp_dir, filename)
                    with open(filepath, "wb") as qf:
                        qf.write(payload)
                        
                    quantized_files[name] = (filename, filepath)
                else:
                    unquantized_tensors[name] = W.to(torch.bfloat16)

    # 4. Save unquantized tensors to safetensors
    unquantized_path = os.path.join(tmp_dir, "model.safetensors")
    save_file(unquantized_tensors, unquantized_path)

    # 5. Build manifest.json
    manifest_files = []
    
    # Add config
    manifest_files.append(manifest_file(config_path, "config.json"))
    
    # Add tokenizer if exists
    tokenizer_path = os.path.join(args.model_dir, "tokenizer.json")
    if os.path.exists(tokenizer_path):
        manifest_files.append(manifest_file(tokenizer_path, "tokenizer.json"))

    # Add unquantized safetensors
    manifest_files.append(manifest_file(unquantized_path, "model.safetensors"))

    # Add quantized tensors
    for name, (filename, filepath) in quantized_files.items():
        manifest_files.append(manifest_file(filepath, filename))

    manifest = {
        "format": "ufo-v2",
        "mode": "quantized",
        "model_name": model_name,
        "quant_mode": args.mode.upper(),
        "files": manifest_files
    }

    manifest_json_path = os.path.join(tmp_dir, "manifest.json")
    with open(manifest_json_path, "w") as mf:
        json.dump(manifest, mf, indent=2)

    # 6. Archive to output zip file
    print(f"\n[*] Creating capsule archive: {args.out_capsule}")
    with zipfile.ZipFile(args.out_capsule, "w", compression=zipfile.ZIP_STORED) as zip_file:
        zip_file.write(manifest_json_path, "manifest.json")
        zip_file.write(config_path, "config.json")
        if os.path.exists(tokenizer_path):
            zip_file.write(tokenizer_path, "tokenizer.json")
        zip_file.write(unquantized_path, "model.safetensors")
        for filename, filepath in quantized_files.values():
            zip_file.write(filepath, filename, compress_type=zipfile.ZIP_STORED)

    # 7. Cleanup temp folder
    for filepath in [manifest_json_path, unquantized_path] + [f[1] for f in quantized_files.values()]:
        if os.path.exists(filepath):
            os.remove(filepath)
    if os.path.exists(tmp_dir):
        os.rmdir(tmp_dir)

    print("\n[+] Executable Quantized Capsule v2 packaged successfully!")
    print(f"    [+] Capsule size: {os.path.getsize(args.out_capsule) / (1024**2):.2f} MB")

if __name__ == "__main__":
    main()
