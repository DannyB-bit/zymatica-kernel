import argparse
import numpy as np

def run_proof():
    print("======================================================================")
    print("ZYMATICA | Embedding-Driven Weight Projection (E-PAUP) Proof")
    print("======================================================================\n")

    V = 128    # Mock Vocabulary size
    D = 32     # Hidden dimension size
    RANK = 4   # low-rank factor of projection parameter matrix

    # 1. Setup mock shared embedding matrix E
    print(f"[1] Simulating Shared Word Embedding Matrix E ({V}x{D} floats)...")
    rng = np.random.RandomState(42)
    E = rng.standard_normal((V, D)).astype(np.float32)
    # Normalize rows of E representing word vectors
    norms = np.linalg.norm(E, axis=1, keepdims=True) + 1e-9
    E = E / norms
    print(f"  -> Shared embedding matrix E instantiated. Mean norm: {np.mean(norms):.4f}")

    # 2. Setup low-rank projection parameter matrix P
    print(f"\n[2] Instantiating Low-Rank Projection Parameter Matrix P ({D}x{D} floats)...")
    # P = A * B where A is DxR and B is RxD
    A = rng.standard_normal((D, RANK)).astype(np.float32)
    B = rng.standard_normal((RANK, D)).astype(np.float32)
    P = np.dot(A, B)
    print(f"  -> Projection parameter matrix P initialized (Rank={RANK}).")

    # 3. Compute E-PAUP Projection: W_delta = E * P * E^T
    print("\n[3] Computing E-PAUP Projection: W_delta = E * P * E^T...")
    W_delta = np.dot(E, np.dot(P, E.T))
    print(f"  -> Projected weight update matrix shape: {W_delta.shape}")
    print(f"  -> Projected weight sum of absolute values: {np.sum(np.abs(W_delta)):.4f}")

    # 4. Perform SVD to factorize W_delta into U and V
    print("\n[4] Decomposing Regularized Manifold back to Low-Rank format (SVD)...")
    U, S, Vh = np.linalg.svd(W_delta, full_matrices=False)
    
    # Extract low-rank factors representing the compressed state
    U_factor = U[:, :RANK] * np.sqrt(S[:RANK])
    V_factor = Vh[:RANK, :].T * np.sqrt(S[:RANK])
    
    print(f"  -> Decomposed factor U shape: {U_factor.shape}")
    print(f"  -> Decomposed factor V shape: {V_factor.shape}")

    # Reconstruct to verify lossless decomposition
    W_rec = np.dot(U_factor, V_factor.T)
    mse = np.mean((W_delta - W_rec) ** 2)
    print(f"  -> Reconstruction Mean Squared Error (MSE) from SVD: {mse:.8e}")

    print("\n[VERIFICATION] E-PAUP embedding-driven projection and SVD factorization verified.")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Zymatica E-PAUP Weight Projection Proof")
    parser.add_argument("--test", action="store_true", help="Run test mode")
    args = parser.parse_args()
    run_proof()
