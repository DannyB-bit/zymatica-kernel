import os
import gc
import json
import time
import argparse
import torch
import numpy as np
from safetensors import safe_open
from safetensors.torch import save_file

def quantize_to_q8(W_active):
    """Quantize weight matrix to 8-bit integers with a float scale factor."""
    max_val = np.max(np.abs(W_active))
    scale = max_val / 127.0 if max_val > 0 else 1e-9
    W_q = np.clip(np.round(W_active / scale), -127, 127).astype(np.int8)
    return torch.from_numpy(W_q), float(scale)

def main():
    parser = argparse.ArgumentParser(description="Gemma-4-E2B SVD/Q8 Compressor")
    parser.add_argument("--model-dir", type=str, required=True, help="Directory containing model.safetensors")
    parser.add_argument("--out-file", type=str, required=True, help="Path to output compressed safetensors")
    parser.add_argument("--attn-rank", type=int, default=64, help="SVD Rank for attention projections")
    parser.add_argument("--mlp-rank", type=int, default=128, help="SVD Rank for MLP projections")
    args = parser.parse_args()

    print("=" * 80)
    print("  GEMMA-4-E2B SVD/Q8 COMPRESSOR")
    print("=" * 80)

    device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
    print(f"[*] SVD Execution Device: {device}")

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
        print(f"[-] Error: Neither safetensors index nor single model.safetensors found in {args.model_dir}")
        return

    print(f"\n[1] Identified {len(shards)} shards to process.")

    compressed_dict = {}
    processed_count = 0
    t_start = time.time()
    manifest_metrics = {}

    for shard_name, param_names in shards.items():
        shard_path = os.path.join(args.model_dir, shard_name)
        print(f"\n[*] Processing shard: {shard_name} ({len(param_names)} parameters)...")

        with safe_open(shard_path, framework="pt", device="cpu") as f:
            for param_name in param_names:
                is_svd = any(proj in param_name for proj in ["q_proj", "k_proj", "v_proj", "o_proj", "gate_proj", "up_proj", "down_proj"])

                try:
                    W = f.get_tensor(param_name)
                except Exception as e:
                    print(f"    [-] Error loading tensor {param_name}: {e}")
                    continue

                if is_svd and W.ndim == 2:
                    # Select SVD rank based on projection type
                    r = args.mlp_rank if any(proj in param_name for proj in ["gate_proj", "up_proj", "down_proj"]) else args.attn_rank
                    
                    # Bound rank to the smaller matrix dimension
                    r = min(r, W.shape[0], W.shape[1])
                    
                    # Run SVD
                    W_dev = W.float().to(device)
                    U, S, Vt = torch.linalg.svd(W_dev, full_matrices=False)
                    
                    # Scale components to absorb singular values
                    U_active = U[:, :r] * torch.sqrt(S[:r])
                    V_active = Vt[:r, :].t() * torch.sqrt(S[:r])
                    
                    U_active_cpu = U_active.cpu().numpy()
                    V_active_cpu = V_active.cpu().numpy()
                    
                    # Quantize singular vectors to Q8
                    U_q, scale_u = quantize_to_q8(U_active_cpu)
                    V_q, scale_v = quantize_to_q8(V_active_cpu)
                    
                    # Compute relative Frobenius reconstruction error
                    U_rec = (U_q.float() * scale_u).to(device)
                    V_rec = (V_q.float() * scale_v).to(device)
                    W_rec = U_rec @ V_rec.t()
                    
                    err_fro = torch.linalg.norm(W_dev - W_rec).item()
                    norm_fro = torch.linalg.norm(W_dev).item()
                    rel_err = err_fro / norm_fro if norm_fro > 0 else 0.0

                    manifest_metrics[param_name] = {
                        "rank": r,
                        "scale_u": scale_u,
                        "scale_v": scale_v,
                        "rel_reconstruction_error": rel_err
                    }
                    
                    # Save components to output dictionary
                    compressed_dict[f"{param_name}.U_q"] = U_q
                    compressed_dict[f"{param_name}.V_q"] = V_q
                    compressed_dict[f"{param_name}.scale_u"] = torch.tensor(scale_u, dtype=torch.float32)
                    compressed_dict[f"{param_name}.scale_v"] = torch.tensor(scale_v, dtype=torch.float32)
                    
                    del W_dev, U, S, Vt, U_active, V_active, U_rec, V_rec, W_rec
                else:
                    # Keep non-projection parameters as-is in bfloat16
                    compressed_dict[param_name] = W.to(torch.bfloat16)
                    
                del W
                gc.collect()
                if device.type == "cuda":
                    torch.cuda.empty_cache()
                
                processed_count += 1
                if processed_count % 50 == 0:
                    print(f"    [Progress] Processed {processed_count} parameters...")
                    
        gc.collect()
        
    print(f"\n[+] Successfully compressed/extracted {processed_count} layers in {time.time() - t_start:.2f}s.")
    
    # Ensure all saved tensors are contiguous in memory to prevent safetensors ValueError
    for k in list(compressed_dict.keys()):
        if isinstance(compressed_dict[k], torch.Tensor):
            compressed_dict[k] = compressed_dict[k].contiguous()
            
    # Save the output file
    out_dir = os.path.dirname(args.out_file)
    if out_dir:
        os.makedirs(out_dir, exist_ok=True)
        
    print(f"\n[2] Writing compressed weights to: {args.out_file}")
    save_file(compressed_dict, args.out_file)
    print(f"    [+] Saved successfully. File size: {os.path.getsize(args.out_file) / (1024**2):.2f} MB")

    # Save manifest file
    manifest_path = args.out_file + ".manifest.json"
    print(f"[3] Writing compression manifest to: {manifest_path}")
    manifest_data = {
        "source_dir": args.model_dir,
        "attn_rank": args.attn_rank,
        "mlp_rank": args.mlp_rank,
        "parameters": manifest_metrics,
        "timestamp": time.strftime("%Y-%m-%d %H:%M:%S")
    }
    with open(manifest_path, "w") as mf:
        json.dump(manifest_data, mf, indent=2)
    print("    [+] Manifest saved successfully.")

if __name__ == "__main__":
    main()
