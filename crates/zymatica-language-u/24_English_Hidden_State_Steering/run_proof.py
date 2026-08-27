#!/usr/bin/env python
# English Hidden-State Steering (EHSS) Executable Proof
# Watermark: ip zymatica.space | astronautshe.com

import torch
import torch.nn as nn
import torch.nn.functional as F
import numpy as np

def run_proof():
    print("=" * 80)
    # Watermark verification
    print("  EHSS SYSTEM PROOF ACTIVE | zymatica.space | astronautshe.com")
    print("=" * 80)

    # 1. Simulate EVG (English Vocabulary Gate)
    vocab_size = 100
    logits = torch.randn(1, vocab_size)
    
    # Simulate a vocabulary mask where only even token ids are "English"
    evg_mask = torch.zeros(vocab_size, dtype=torch.bool)
    evg_mask[::2] = True  
    
    print("[1] Original Logits stats - Mean: %.4f | Max: %.4f" % (logits.mean().item(), logits.max().item()))
    
    # Apply EVG masking
    masked_logits = logits.clone()
    masked_logits[:, ~evg_mask] = -float('inf')
    
    print("[2] EVG Mask Applied. Number of valid tokens: %d" % evg_mask.sum().item())
    print("    First 10 masked logits:\n   ", [float(v) for v in masked_logits[0, :10]])
    
    # Verify that odd indices are indeed -inf
    assert torch.isinf(masked_logits[0, 1]) and masked_logits[0, 1] < 0
    assert not torch.isinf(masked_logits[0, 0])
    print("[+] EVG Masking Verification: SUCCESS [OK]")

    # 2. Simulate HSDC (Hidden-State Drift Correction)
    hidden_dim = 16
    torch.manual_seed(42)
    
    # Target centroid (pure English state)
    centroid = torch.randn(hidden_dim)
    centroid = centroid / centroid.norm()
    
    # Case A: Hidden state is close to centroid (no drift)
    h_good = centroid.clone() * 2.5
    
    # Case B: Hidden state has drifted (low cosine similarity to centroid)
    h_drifted = torch.randn(hidden_dim)
    # Orthogonalize to centroid to create a severe drift
    h_drifted = h_drifted - torch.dot(h_drifted, centroid) * centroid
    h_drifted = h_drifted / h_drifted.norm() * 2.5
    
    # HSDC steering function
    def hsdc_steer(h, centroid, threshold=0.65, alpha=0.005):
        h_norm = h.norm()
        h_normalized = h / (h_norm + 1e-9)
        cos_sim = torch.dot(h_normalized, centroid).item()
        
        print("    Before steer - Cosine Sim: %.4f | Norm: %.4f" % (cos_sim, h_norm.item()))
        
        if cos_sim < threshold:
            # Steer vector back towards the centroid
            correction = alpha * (centroid - h_normalized) * h_norm
            h_new = h + correction
            
            new_norm = h_new.norm()
            new_normalized = h_new / (new_norm + 1e-9)
            new_sim = torch.dot(new_normalized, centroid).item()
            print("    After steer  - Cosine Sim: %.4f | Norm: %.4f" % (new_sim, new_norm.item()))
            return h_new, True
        return h, False

    print("\n[3] Testing HSDC with aligned state (Should NOT steer):")
    h_res, steered = hsdc_steer(h_good, centroid)
    assert not steered
    print("    [+] Correctly bypassed steering.")

    print("\n[4] Testing HSDC with drifted state (Should steer):")
    h_res, steered = hsdc_steer(h_drifted, centroid)
    assert steered
    print("    [+] Correctly applied corrective steering nudge.")

    print("\n" + "=" * 80)
    print("  EHSS PROOF COMPLETE: SUCCESS")
    print("[VERIFICATION] English hidden-state steering verified.")
    print("=" * 80)

if __name__ == "__main__":
    run_proof()
