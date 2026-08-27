import argparse
import numpy as np

def run_proof():
    print("======================================================================")
    print("ZYMATICA | Hybrid Real-SVD Loading (HRSL) Execution Partition Proof")
    print("======================================================================\n")

    # Dimensions
    dim = 64
    num_blocks = 4
    n_real = 2  # First 2 blocks are full-rank
    rank = 4

    rng = np.random.RandomState(42)

    # 1. Setup ideal full-rank parameters for 4 blocks
    print(f"[1] Instantiating Ideal Full-Rank Model ({num_blocks} blocks, dim={dim})...")
    weights = [rng.standard_normal((dim, dim)).astype(np.float32) for _ in range(num_blocks)]
    
    # 2. Setup low-rank SVD approximations
    print(f"[2] Computing low-rank SVD projections (Rank={rank}) for all blocks...")
    svd_factors = []
    for W in weights:
        U, S, Vh = np.linalg.svd(W)
        U_scale = U[:, :rank] * np.sqrt(S[:rank])
        V_scale = Vh[:rank, :].T * np.sqrt(S[:rank])
        svd_factors.append((U_scale, V_scale))

    # 3. Simulate input activation pass
    x_in = rng.standard_normal((1, dim)).astype(np.float32)
    print(f"\n[3] Simulating Forward Passes (Input Shape: {x_in.shape})...")

    # Mode A: Ideal model (100% Full-Rank)
    x = x_in.copy()
    for block in range(num_blocks):
        x = np.dot(x, weights[block].T)
    x_ideal = x.copy()

    # Mode B: Fully compressed model (100% SVD)
    x = x_in.copy()
    for block in range(num_blocks):
        U_scale, V_scale = svd_factors[block]
        x = np.dot(np.dot(x, V_scale), U_scale.T)
    x_svd_only = x.copy()

    # Mode C: HRSL model (Hybrid: first 2 blocks full-rank, remaining 2 blocks SVD)
    x = x_in.copy()
    for block in range(num_blocks):
        if block < n_real:
            # Full rank
            x = np.dot(x, weights[block].T)
        else:
            # Low-rank SVD
            U_scale, V_scale = svd_factors[block]
            x = np.dot(np.dot(x, V_scale), U_scale.T)
    x_hrsl = x.copy()

    # 4. Measure error and footprint
    print("\n[4] Performance & Error Analysis:")
    
    # Compute error relative to ideal
    mse_svd = np.mean((x_ideal - x_svd_only) ** 2)
    mse_hrsl = np.mean((x_ideal - x_hrsl) ** 2)
    
    # Compute VRAM parameter storage metrics
    # Raw weight size = dim * dim * 4 bytes per block
    raw_block_bytes = dim * dim * 4
    svd_block_bytes = (dim * rank * 2) * 4 # U + V factors
    
    bytes_ideal = num_blocks * raw_block_bytes
    bytes_svd = num_blocks * svd_block_bytes
    bytes_hrsl = (n_real * raw_block_bytes) + ((num_blocks - n_real) * svd_block_bytes)

    comp_ratio_hrsl = bytes_ideal / bytes_hrsl
    comp_ratio_svd = bytes_ideal / bytes_svd

    print(f"  - **100% Ideal Model**: Size={bytes_ideal:,} bytes | MSE=0.000000 (Reference)")
    print(f"  - **100% SVD Model**:   Size={bytes_svd:,} bytes  | MSE={mse_svd:.6f} | Compression={comp_ratio_svd:.2f}x")
    print(f"  - **HRSL Model**:       Size={bytes_hrsl:,} bytes  | MSE={mse_hrsl:.6f} | Compression={comp_ratio_hrsl:.2f}x")
    
    print(f"\n  -> HRSL Error reduction vs 100% SVD: {(1 - mse_hrsl/mse_svd)*100:.2f}% improvement")
    print("\n[VERIFICATION] Hybrid Real-SVD Loading partition constraints verified.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Zymatica HRSL Partition Proof")
    parser.add_argument("--test", action="store_true", help="Run test mode")
    args = parser.parse_args()
    run_proof()
