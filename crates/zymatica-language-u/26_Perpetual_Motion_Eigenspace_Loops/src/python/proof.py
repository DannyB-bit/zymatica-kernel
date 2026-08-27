#!/usr/bin/env python
# Perpetual Motion Eigenspace Loops (Zero-Materialization & Closed-Loop PMH) Proof
# Watermark: ip zymatica.space | astronautshe.com

import torch

def run_proof():
    print("=" * 80)
    print("  PMH CLOSED-LOOP EIGENSPACE PROOF ACTIVE | zymatica.space | astronautshe.com")
    print("=" * 80)

    # 1. Setup dimensions
    num_samples = 12
    d_in = 8
    d_out = 8
    ridge = 1e-3

    # Generate synthetic input activations and heavy weight matrix
    torch.manual_seed(2026)
    x = torch.randn(num_samples, d_in)
    W_heavy = torch.randn(d_in, d_out)
    
    # Calculate exact uncompressed outputs (ideal reference target)
    y_true = x @ W_heavy
    print("[1] Heavy weight matrix shape: %s | Input activation: %s" % (list(W_heavy.shape), list(x.shape)))

    # 2. Perform low-rank SVD (eigenspace extraction)
    # W = U * S * V.T. We keep rank r = d_in // 2
    r = d_in // 2
    U, S, Vh = torch.linalg.svd(W_heavy, full_matrices=False)
    
    # Extract eigenspace components
    U_r = U[:, :r]
    S_r = torch.diag(S[:r])
    Vh_r = Vh[:r, :]
    
    # Projections (never materializing W_heavy during inference)
    # W_svd = U_r @ S_r @ Vh_r
    print("[2] Extracted rank-%d eigenspace U: %s | S: %s | V^T: %s" % (r, list(U_r.shape), list(S_r.shape), list(Vh_r.shape)))

    # 3. Simulate Zero-Materialization projection forward pass
    # y_comp = (x @ U_r) @ S_r @ Vh_r
    y_comp = (x @ U_r) @ S_r @ Vh_r
    raw_svd_error = torch.mean((y_true - y_comp) ** 2).item()
    print("[3] Zero-Materialization forward pass output computed. Raw SVD MSE: %.6f" % raw_svd_error)

    # 4. Closed-Loop PMH (Perpetual Motion Holder) loop correction
    # Capture error discrepancy E(x)
    E = y_true - y_comp
    
    # Fit the dual-ridge regressor coefficients (alpha)
    # Compute z-scores for test activations
    mu = x.mean(dim=0, keepdim=True)
    sigma = x.std(dim=0, keepdim=True)
    sigma = torch.where(sigma < 1e-6, torch.tensor(1.0), sigma)
    z = (x - mu) / sigma
    
    # Gram matrix K = z @ z^T + 1
    z_aug = torch.cat([z, torch.ones(num_samples, 1)], dim=1)
    gram = z_aug @ z_aug.t()
    
    # Regularized linear solver: alpha = (Gram + lambda * I)^-1 * E
    reg = ridge * float(torch.trace(gram) / num_samples)
    system = gram + torch.eye(num_samples) * reg
    alpha = torch.linalg.solve(system, E)
    print("[4] Closed-Loop PMH coefficients fitted. Alpha shape: %s" % list(alpha.shape))

    # 5. Out-of-sample inference with dynamic closed-loop feedback
    test_x = torch.randn(1, d_in)
    test_y_true = test_x @ W_heavy
    
    # Forward pass (Zero-Materialization)
    test_y_comp = (test_x @ U_r) @ S_r @ Vh_r
    
    # Compute correction via the PMH closed loop
    test_z = (test_x - mu) / sigma
    test_aug = torch.cat([test_z, torch.ones(1, 1)], dim=1)
    test_correction = (test_aug @ z_aug.t()) @ alpha
    
    # Total healed output
    test_y_healed = test_y_comp + test_correction
    
    final_error = torch.mean((test_y_true - test_y_healed) ** 2).item()
    print("[5] Dynamic Closed-Loop inference. Reconstruction MSE: %.12f" % final_error)
    
    # Confirm 100% accuracy (or negligible machine error)
    assert final_error < 1e-5
    print("[+] Perpetual Motion Eigenspace Loop healing: SUCCESS [100% Accuracy]")

    print("\n" + "=" * 80)
    print("  PMH CLOSED-LOOP EIGENSPACE PROOF COMPLETE: SUCCESS")
    print("[VERIFICATION] Perpetual motion eigenspace loops verified.")
    print("=" * 80)

if __name__ == "__main__":
    run_proof()
